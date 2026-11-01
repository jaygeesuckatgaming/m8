use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub mate: MateConfig,
    pub mic: MicConfig,
    pub stt: SttConfig,
    pub kev: KevConfig,
    pub llm: LlmConfig,
    pub models: ModelsConfig,
    pub game: GameConfig,
    /// What the mate remembers between games; off if the section is missing.
    #[serde(default)]
    pub memory: MemoryConfig,
    /// Folder the config file lives in; relative paths are resolved against it.
    #[serde(skip)]
    pub base_dir: PathBuf,
}

#[derive(Deserialize, Clone)]
pub struct MateConfig {
    pub name: String,
    /// The persona's folder: system_prompt.txt (who the mate is), combat_prompt.txt and
    /// radio_prompt.txt (added while the game says so; either may be missing) and levels/ (each
    /// level's story, by the name the game gives it). The config's folder if unset.
    #[serde(default)]
    pub persona: Option<PathBuf>,
    /// The voice: a server with Breeze TTS 2's API (speak.rs; start-breeze.ps1)...
    pub voice_server: String,
    /// ...and the WAV it clones; its transcript is the .txt beside it.
    pub voice_reference: PathBuf,
    /// Its noise seed, so every line comes out in the same voice.
    pub voice_seed: u64,
}

#[derive(Deserialize, Clone)]
pub struct MicConfig {
    /// Input device name (case-insensitive substring); the system default if unset.
    #[serde(default)]
    pub device: Option<String>,
    /// Talking over the mate cuts it off. Takes precedence over `mute_while_speaking`.
    #[serde(default = "yes")]
    pub barge_in: bool,
    pub mute_while_speaking: bool,
    /// Save every recognised clip to `recordings/` for debugging.
    #[serde(default)]
    pub save_recordings: bool,
    pub vad_threshold: f32,
    pub end_of_speech_silence: f32,
}

/// Speech to text: an OpenAI-style /v1/audio/transcriptions (listen.rs).
#[derive(Deserialize, Clone)]
pub struct SttConfig {
    pub url: String,
    /// The `model` it's sent, for servers that want one.
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct KevConfig {
    pub url: String,
}

#[derive(Deserialize, Clone)]
pub struct LlmConfig {
    pub url: String,
    /// llama.cpp ignores this; OpenAI-compatible servers (Ollama, vLLM) need it.
    #[serde(default)]
    pub model: Option<String>,
    /// Optional separate model for `Brain::look` (the mate's eyes); empty uses `model`.
    #[serde(default)]
    pub vision_model: Option<String>,
    pub temperature: f32,
}

#[derive(Deserialize, Clone)]
pub struct ModelsConfig {
    /// Silero VAD, which finds where speech starts and ends in the mic's audio.
    pub silero_vad: PathBuf,
}

#[derive(Deserialize, Clone)]
pub struct GameConfig {
    pub port: u16,
}

/// The memory between games: the Gem system's Cognee server (its HTTP API, :8011), the same
/// memory the stream's Gem has. Its entries read "[halo] Gem: ..." beside the chat's.
#[derive(Deserialize, Clone)]
pub struct MemoryConfig {
    /// The Cognee server; unset turns memory off.
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default = "yes")]
    pub enabled: bool,
    /// The tag the entries carry.
    #[serde(default = "halo_source")]
    pub source: String,
    /// How many seconds of notable events are gathered into one entry.
    #[serde(default = "default_flush_seconds")]
    pub flush_seconds: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self { url: None, enabled: true, source: halo_source(), flush_seconds: default_flush_seconds() }
    }
}

fn halo_source() -> String {
    "halo".to_string()
}

fn default_flush_seconds() -> u64 {
    30
}

impl Config {
    /// Loads the TOML file and makes every model path absolute relative to it.
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading config {}", path.display()))?;
        let mut cfg: Config = toml::from_str(&text).context("parsing config")?;
        let base = path.parent().unwrap_or(Path::new("."));
        cfg.base_dir = base.to_path_buf();
        for p in [&mut cfg.models.silero_vad, &mut cfg.mate.voice_reference].into_iter().chain(cfg.mate.persona.as_mut()) {
            if p.is_relative() {
                *p = base.join(&*p);
            }
        }
        Ok(cfg)
    }
}

impl Config {
    /// The persona's folder (`persona`), else the config's.
    pub fn persona_dir(&self) -> PathBuf {
        self.mate.persona.clone().unwrap_or_else(|| self.base_dir.clone())
    }
}

fn yes() -> bool {
    true
}

/// sherpa-onnx wants paths as `String`s.
pub fn path_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}
