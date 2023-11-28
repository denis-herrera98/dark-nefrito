use dashmap::DashMap;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::{Mutex};
use lazy_static::lazy_static;
use vosk::{Model, Recognizer};
use hound::WavReader;
use std::fs::File;
use std::io::{self, Write, BufReader};
use std::sync::RwLock;
use hound::WavWriter;

use std::{
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            Args,
            CommandResult,
        },
        StandardFramework,
    },
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::{GatewayIntents, Mentionable},
    Result as SerenityResult,
};

use songbird::{
    driver::DecodeMode,
    model::{
        id::UserId,
        payload::{ClientDisconnect, Speaking},
    },
    packet::Packet,
    Config,
    CoreEvent,
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
    SerenityInit,
};

type AudioData = Vec<i16>;

pub struct VoiceDataManager {
    all_audio: HashMap<u32, AudioData>,
}

impl VoiceDataManager {
    
    fn new() -> VoiceDataManager {
        VoiceDataManager {
            all_audio: HashMap::new()
        } 
    }

    fn instance() -> &'static Mutex<VoiceDataManager> {
        static INSTANCE: Lazy<Mutex<VoiceDataManager>> = Lazy::new(|| {
            Mutex::new(VoiceDataManager::new())
        });
        
        &INSTANCE
    }

    pub fn get_instance() -> &'static Mutex<VoiceDataManager> {
        Self::instance()
    }

    pub fn add_audio(&mut self, user: u32, audio: Vec<i16>) {

        if !self.all_audio.contains_key(&user) {
            self.all_audio.insert(user, audio);
        }

//        if let Some(user_prev_data) = self.all_audio.get_mut(&user) {
//            user_prev_data.extend(audio);
//        } 
    }
}


//lazy_static! {
//    static ref ALL_AUDIO: RwLock<HashMap<u32, AudioData>> = RwLock::new(HashMap::new());
//}


pub struct ModelManager {
    recognizer: Mutex<Recognizer>,
}

impl ModelManager {
    fn new() -> Self {
        let model_path = "/home/denisherrera/Documents/models/vosk-model-es-0.42";
        let model = Model::new(model_path).expect("Failed to load model");
        let mut recognizer = Recognizer::new(&model, 96000.0).unwrap();
        recognizer.set_max_alternatives(3);
        recognizer.set_words(true);
        recognizer.set_partial_words(true);

        Self {
            recognizer: Mutex::new(recognizer),
        }
    }

    pub fn get_instance() -> &'static Mutex<Recognizer> {
        lazy_static! {
            static ref INSTANCE: ModelManager = ModelManager::new();
        }

        &INSTANCE.recognizer
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[derive(Clone)]
struct Receiver {
    inner: Arc<InnerReceiver>,
}

struct InnerReceiver {
    last_tick_was_empty: AtomicBool,
    known_ssrcs: DashMap<u32, UserId>,
    users_audio: DashMap<String, AudioData>,
}

impl Receiver {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        Self {
            inner: Arc::new(InnerReceiver {
                last_tick_was_empty: AtomicBool::default(),
                known_ssrcs: DashMap::new(),
                users_audio: DashMap::new(),
            }),
        }
    }
}

#[async_trait]
impl VoiceEventHandler for Receiver {
    #[allow(unused_variables)]
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ctx {
            Ctx::SpeakingStateUpdate(Speaking {
                speaking,
                ssrc,
                user_id,
                ..
            }) => {
                // Discord voice calls use RTP, where every sender uses a randomly allocated
                // *Synchronisation Source* (SSRC) to allow receivers to tell which audio
                // stream a received packet belongs to. As this number is not derived from
                // the sender's user_id, only Discord Voice Gateway messages like this one
                // inform us about which random SSRC a user has been allocated. Future voice
                // packets will contain *only* the SSRC.
                //
                // You can implement logic here so that you can differentiate users'
                // SSRCs and map the SSRC to the User ID and maintain this state.
                // Using this map, you can map the `ssrc` in `voice_packet`
                // to the user ID and handle their audio packets separately.
                println!(
                    "Speaking state update: user {:?} has SSRC {:?}, using {:?}",
                    user_id, ssrc, speaking,
                );

                if let Some(user) = user_id {
                    println!("ADDING user_id: {}, ssrc: {} to know users", user, ssrc);
                    self.inner.known_ssrcs.insert(*ssrc, *user);
                }
            },
            Ctx::VoiceTick(tick) => {
                let speaking = tick.speaking.len();
                let total_participants = speaking + tick.silent.len();
                let last_tick_was_empty = self.inner.last_tick_was_empty.load(Ordering::SeqCst);

                if speaking == 0 && !last_tick_was_empty {
                    println!("No speakers");

                    self.inner.last_tick_was_empty.store(true, Ordering::SeqCst);
                } else if speaking != 0 {
                    self.inner
                        .last_tick_was_empty
                        .store(false, Ordering::SeqCst);

                    // println!("Voice tick ({speaking}/{total_participants} live):");

                    // You can also examine tick.silent to see users who are present
                    // but haven't spoken in this tick.
                    for (ssrc, data) in &tick.speaking {
                        let user_id_str = if let Some(id) = self.inner.known_ssrcs.get(ssrc) {
                            format!("{}", *id)
                        } else {
                            "?".into()
                        };

                        // This field should *always* exist under DecodeMode::Decode.
                        // The `else` allows you to see how the other modes are affected.
                        if let Some(decoded_voice) = data.decoded_voice.as_ref() {
                            let voice_len = decoded_voice.len();
                            let audio_str = format!(
                                "first samples from {}: {:?}",
                                voice_len,
                                &decoded_voice[..voice_len.min(5)]
                            );
                            

                            println!("Inserting data for {}", user_id_str);
                            self.inner.users_audio.entry(user_id_str.clone()).or_insert_with(Vec::new).extend(decoded_voice.to_owned());

                            if let Some(packet) = &data.packet {
                                let rtp = packet.rtp();
                              //  println!(
                              //      "\t{ssrc}/{user_id_str}: packet seq {} ts {} -- {audio_str}",
                              //      rtp.get_sequence().0,
                              //      rtp.get_timestamp().0
                              //  );
                            } else {
                                println!("\t{ssrc}/{user_id_str}: Missed packet -- {audio_str}");
                            }
                        } else {
                            println!("\t{ssrc}/{user_id_str}: Decode disabled.");
                        }
                    }
                }
            },
            Ctx::RtpPacket(packet) => {
                // An event which fires for every received audio packet,
                // containing the decoded data.
                let rtp = packet.rtp();
//                println!(
//                    "Received voice packet from SSRC {}, sequence {}, timestamp {} -- {}B long",
//                    rtp.get_ssrc(),
//                    rtp.get_sequence().0,
//                    rtp.get_timestamp().0,
//                    rtp.payload().len()
//                );
            },
            Ctx::RtcpPacket(data) => {
                // An event which fires for every received rtcp packet,
                // containing the call statistics and reporting information.
                println!("RTCP packet received: {:?}", data.packet);
            },
            Ctx::ClientDisconnect(ClientDisconnect { user_id, .. }) => {
                // You can implement your own logic here to handle a user who has left the
                // voice channel e.g., finalise processing of statistics etc.
                // You will typically need to map the User ID to their SSRC; observed when
                // first speaking.

                println!("Client disconnected: user {:?}", user_id);

                if let Some(audio_data_ref) = self.inner.users_audio.get(&user_id.to_string()) {
                    let audio_data = audio_data_ref.value();
                    println!("STARTING WRITING FILE");
                    let mut wav_write = WavWriter::create(format!("/home/denisherrera/Documents/alma-negra/data/{}.wav", user_id),
                        hound::WavSpec {
                        bits_per_sample: 16,
                        channels: 1,
                        sample_format: hound::SampleFormat::Int,
                        sample_rate: 96000,
                    }).unwrap();
                    for sample in audio_data.iter() {
                        wav_write.write_sample(*sample).ok();
                    }
                    wav_write.finalize().ok();
                    println!("FINALIZED WRITING FILE");
                } else {
                    println!("User {} data was not found", &user_id);
                }

            },
            _ => {
                // We won't be registering this struct for any more event classes.
                unimplemented!()
            },
        }

        None
    }
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let Ok(connect_to) = args.single::<ChannelId>() else {
        check_msg(
            msg.reply(ctx, "Requires a valid voice channel ID be given")
                .await,
        );

        return Ok(());
    };

    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        // NOTE: this skips listening for the actual connection result.
        let mut handler = handler_lock.lock().await;

        let evt_receiver = Receiver::new();

        handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), evt_receiver.clone());
        handler.add_global_event(CoreEvent::RtpPacket.into(), evt_receiver.clone());
        handler.add_global_event(CoreEvent::RtcpPacket.into(), evt_receiver.clone());
        handler.add_global_event(CoreEvent::ClientDisconnect.into(), evt_receiver.clone());
        handler.add_global_event(CoreEvent::VoiceTick.into(), evt_receiver);

        check_msg(
            msg.channel_id
                .say(&ctx.http, &format!("Joined {}", connect_to.mention()))
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Error joining the channel")
                .await,
        );
    }

    Ok(())
}


/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);


       // for sample in read_lock.chunks(2000) {
       //     recognizer.accept_waveform(sample);
       // }

       // let result = recognizer.final_result().multiple().unwrap();
       // let best_alternative = &result.alternatives[0];
       // let text = best_alternative.text;
       
       // check_msg(msg.reply(ctx, text).await);
       // check_msg(msg.channel_id.say(&ctx.http,"text").await);
       // check_msg(msg.reply(ctx, text).await));
       //  println!("{:#?}", &result.alternatives);
       // println!("{}", text);

    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}
