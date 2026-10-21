use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Mutex, OnceLock};

#[cfg(not(target_arch = "wasm32"))]
static OWNED_SERVER: OnceLock<Mutex<Option<std::process::Child>>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn find_command(name: &str) -> Option<PathBuf> {
    let mut directories = std::env::var_os("PATH").map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>()).unwrap_or_default();
    if let Some(home) = std::env::var_os("HOME") {
        directories.push(PathBuf::from(home).join(".local/bin"));
    }
    directories.extend([PathBuf::from("/opt/homebrew/bin"), PathBuf::from("/usr/local/bin"), PathBuf::from("/usr/bin")]);
    let executable = if cfg!(windows) { format!("{name}.exe") } else { name.to_string() };
    directories.into_iter().map(|directory| directory.join(&executable)).find(|path| path.is_file())
}

#[cfg(not(target_arch = "wasm32"))]
fn bundled_uv_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from).ok_or_else(|| "Não foi possível localizar a pasta do usuário para preparar o Auto-Oto.".to_string())?;
    let executable = if cfg!(windows) { "uv.exe" } else { "uv" };
    Ok(home.join(".copaiba").join("mvt-runtime").join(executable))
}

#[cfg(not(target_arch = "wasm32"))]
fn install_uv(progress: &impl Fn(String)) -> Result<PathBuf, String> {
    let destination = bundled_uv_path()?;
    if destination.is_file() { return Ok(destination); }

    let (target, archive_name) = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => ("aarch64-apple-darwin", "tar.gz"),
        ("macos", "x86_64") => ("x86_64-apple-darwin", "tar.gz"),
        ("linux", "aarch64") => ("aarch64-unknown-linux-gnu", "tar.gz"),
        ("linux", "x86_64") => ("x86_64-unknown-linux-gnu", "tar.gz"),
        ("windows", "x86_64") => ("x86_64-pc-windows-msvc", "zip"),
        _ => return Err(format!("O Auto-Oto ainda não prepara o runtime neste sistema ({} {}).", std::env::consts::OS, std::env::consts::ARCH)),
    };
    progress("Baixando o runtime necessário para preparar o mVocalToolkit…".into());
    let http = reqwest::blocking::Client::builder().user_agent("Copaiba-NEO Auto-Oto")
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(60)).build()
        .map_err(|e| format!("Não foi possível preparar o download do runtime: {e}"))?;
    let release: serde_json::Value = http.get("https://api.github.com/repos/astral-sh/uv/releases/latest").send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|e| format!("Não foi possível consultar o runtime uv: {e}"))?
        .json().map_err(|e| format!("Resposta inválida ao consultar o runtime uv: {e}"))?;
    let expected = format!("uv-{target}.{archive_name}");
    let url = release.get("assets").and_then(|v| v.as_array()).and_then(|assets| assets.iter().find(|asset| {
        asset.get("name").and_then(|v| v.as_str()) == Some(expected.as_str())
    })).and_then(|asset| asset.get("browser_download_url")).and_then(|v| v.as_str())
        .ok_or_else(|| format!("O runtime uv não oferece um pacote compatível ({target})."))?;
    let archive = http.get(url).send().and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|e| format!("Falha ao baixar o runtime uv: {e}"))?.bytes()
        .map_err(|e| format!("Falha ao receber o runtime uv: {e}"))?;
    let temp = tempfile::tempdir().map_err(|e| format!("Não foi possível criar a pasta temporária do runtime: {e}"))?;
    let archive_path = temp.path().join(format!("uv.{archive_name}"));
    std::fs::write(&archive_path, &archive).map_err(|e| format!("Não foi possível gravar o runtime temporário: {e}"))?;
    progress("Instalando o runtime do Auto-Oto…".into());
    let extraction = if cfg!(windows) {
        std::process::Command::new("powershell").args(["-NoProfile", "-Command", "Expand-Archive", "-Force", archive_path.to_string_lossy().as_ref(), temp.path().to_string_lossy().as_ref()]).status()
    } else {
        std::process::Command::new("tar").args(["-xzf", archive_path.to_string_lossy().as_ref(), "-C", temp.path().to_string_lossy().as_ref()]).status()
    }.map_err(|e| format!("Não foi possível extrair o runtime uv: {e}"))?;
    if !extraction.success() { return Err("A extração do runtime uv falhou.".into()); }
    let executable = if cfg!(windows) { "uv.exe" } else { "uv" };
    let extracted = temp.path().join(format!("uv-{target}")).join(executable);
    if !extracted.is_file() { return Err("O pacote baixado não contém o executável uv esperado.".into()); }
    let parent = destination.parent().ok_or_else(|| "Caminho de instalação do runtime inválido.".to_string())?;
    std::fs::create_dir_all(parent).map_err(|e| format!("Não foi possível criar a pasta do runtime: {e}"))?;
    std::fs::copy(&extracted, &destination).map_err(|e| format!("Não foi possível instalar o runtime uv: {e}"))?;
    Ok(destination)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn shutdown_owned_service() {
    if let Ok(client) = Client::new("http://127.0.0.1:8765") { client.stop_owned_service(); }
}

#[cfg(not(target_arch = "wasm32"))]
fn owned_server() -> &'static Mutex<Option<std::process::Child>> {
    OWNED_SERVER.get_or_init(|| Mutex::new(None))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MvtSettings {
    pub endpoint: String,
    pub language: String,
    pub align_model: String,
    pub refine_model: String,
    pub margin_ms: f64,
    pub oto_scheme: MvtOtoScheme,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MvtOtoScheme {
    Cv,
    Vcv,
    Cvvc,
    #[default]
    Mixed,
}

impl Default for MvtSettings {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:8765".into(),
            language: "ja".into(),
            align_model: String::new(),
            refine_model: String::new(),
            margin_ms: 10.0,
            oto_scheme: MvtOtoScheme::Mixed,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MvtModel {
    pub id: String,
    pub name: String,
    pub engine: String,
    pub installed: bool,
    pub size_bytes: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct MvtLanguage {
    pub code: String,
    pub name: String,
    pub native_name: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MvtOtoValues {
    pub offset: f64,
    pub consonant: f64,
    pub cutoff: f64,
    pub preutter: f64,
    pub overlap: f64,
}

#[derive(Clone, Debug)]
pub struct MvtOtoProposal {
    pub entry_index: usize,
    pub new_entry: bool,
    pub filename: String,
    pub alias: String,
    pub proposed_alias: Option<String>,
    pub values: Option<MvtOtoValues>,
    pub apply_fields: [bool; 5],
    pub selected: bool,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct MvtWorkItem {
    pub entry_index: usize,
    pub path: PathBuf,
    pub filename: String,
    pub alias: String,
    pub propose: bool,
    pub offset_ms: f64,
    pub preutter_ms: f64,
    pub vcv_style: bool,
    pub vc_style: bool,
    pub generate_vcv_units: bool,
    pub scheme: MvtOtoScheme,
}

pub enum MvtUiEvent {
    Models { language: String, result: Result<(Vec<MvtLanguage>, Vec<MvtModel>), String> },
    Progress { message: String, fraction: Option<f32> },
    Proposals(Result<Vec<MvtOtoProposal>, String>),
}

pub struct MvtUiState {
    pub receiver: Option<Receiver<MvtUiEvent>>,
    pub align_models: Vec<MvtModel>,
    pub languages: Vec<MvtLanguage>,
    pub models_language: Option<String>,
    pub proposals: Vec<MvtOtoProposal>,
    pub status: String,
    pub progress: Option<f32>,
    pub running: bool,
    pub refreshing_models: bool,
    pub cancel: Option<Arc<AtomicBool>>,
    pub source_oto_path: Option<PathBuf>,
}

impl Default for MvtUiState {
    fn default() -> Self {
        Self {
            receiver: None,
            align_models: Vec::new(),
            languages: Vec::new(),
            models_language: None,
            proposals: Vec::new(),
            status: String::new(),
            progress: None,
            running: false,
            refreshing_models: false,
            cancel: None,
            source_oto_path: None,
        }
    }
}

impl MvtUiState {
    pub fn drain_events(&mut self) {
        let Some(receiver) = self.receiver.as_ref() else { return; };
        while let Ok(event) = receiver.try_recv() {
            match event {
                MvtUiEvent::Models { language, result: Ok((languages, align)) } => {
                    self.languages = languages;
                    self.align_models = align;
                    self.models_language = Some(language);
                    self.status = format!("{} modelos de alinhamento disponíveis para este idioma", self.align_models.len());
                    self.refreshing_models = false;
                    self.cancel = None;
                }
                MvtUiEvent::Models { language, result: Err(error) } => {
                    self.models_language = Some(language);
                    self.status = error;
                    self.refreshing_models = false;
                    self.cancel = None;
                }
                MvtUiEvent::Progress { message, fraction } => {
                    self.status = message;
                    self.progress = fraction.map(|value| value.clamp(0.0, 1.0));
                }
                MvtUiEvent::Proposals(Ok(proposals)) => {
                    let count = proposals.iter().filter(|p| p.values.is_some() || p.proposed_alias.is_some()).count();
                    self.proposals = proposals;
                    self.running = false;
                    self.cancel = None;
                    self.progress = None;
                    self.status = format!("Análise concluída: {count} proposta(s) disponível(is). Revise antes de aplicar.");
                }
                MvtUiEvent::Proposals(Err(error)) => {
                    self.running = false;
                    self.cancel = None;
                    self.progress = None;
                    self.status = error;
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn refresh_models(settings: MvtSettings, ui: &mut MvtUiState) {
    if ui.running || ui.refreshing_models { return; }
    let (tx, rx) = mpsc::channel();
    ui.receiver = Some(rx);
    let cancel = Arc::new(AtomicBool::new(false));
    ui.cancel = Some(cancel.clone());
    ui.refreshing_models = true;
    ui.progress = None;
    ui.status = "Consultando mVocalToolkit…".into();
    std::thread::spawn(move || {
        let result = Client::new(&settings.endpoint).and_then(|client| {
            client.ensure_local_service(&cancel, |message| { let _ = tx.send(MvtUiEvent::Progress { message, fraction: None }); })?;
            if cancel.load(Ordering::Relaxed) { return Err("Preparação do mVocalToolkit cancelada.".into()); }
            client.health()?;
            let languages = client.languages()?;
            let align = client.models("align", Some(&settings.language))?;
            Ok((languages, align))
        });
        let _ = tx.send(MvtUiEvent::Models { language: settings.language, result });
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start_analysis(settings: MvtSettings, items: Vec<MvtWorkItem>, ui: &mut MvtUiState) {
    if ui.running || ui.refreshing_models { return; }
    if items.is_empty() {
        ui.status = "Selecione uma ou mais entradas com WAV disponível.".into();
        return;
    }
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    ui.receiver = Some(rx);
    ui.cancel = Some(cancel.clone());
    ui.running = true;
    ui.progress = None;
    ui.proposals.clear();
    ui.status = "Preparando alinhamento…".into();
    std::thread::spawn(move || {
        let progress_tx = tx.clone();
        let progress = move |message: String, fraction: Option<f32>| { let _ = progress_tx.send(MvtUiEvent::Progress { message, fraction }); };
        let result = run_analysis(settings, items, cancel, &progress);
        let _ = tx.send(MvtUiEvent::Proposals(result));
    });
}

#[cfg(not(target_arch = "wasm32"))]
struct Client {
    base: String,
    http: reqwest::blocking::Client,
}

#[cfg(not(target_arch = "wasm32"))]
impl Client {
    fn new(_endpoint: &str) -> Result<Self, String> {
        let base = "http://127.0.0.1:8765".to_string();
        let http = reqwest::blocking::Client::builder()
            // The toolkit is a loopback service managed by Copaiba. Do not send
            // health/model/job requests through HTTP_PROXY: managed environments
            // may route those to a network proxy that blocks private addresses.
            .no_proxy()
            .connect_timeout(std::time::Duration::from_secs(3))
            .timeout(std::time::Duration::from_secs(35))
            .build().map_err(|e| format!("Não foi possível criar o cliente HTTP: {e}"))?;
        Ok(Self { base, http })
    }

    fn health(&self) -> Result<(), String> {
        self.http.get(format!("{}/health", self.base))
            .timeout(std::time::Duration::from_secs(1))
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map(|_| ())
            .map_err(|e| format!("mVocalToolkit indisponível em {}: {e}", self.base))
    }

    fn ensure_local_service<F>(&self, cancel: &AtomicBool, progress: F) -> Result<(), String>
    where F: Fn(String) {
        if cancel.load(Ordering::Relaxed) { return Err("Preparação do mVocalToolkit cancelada.".into()); }
        if self.health().is_ok() { return Ok(()); }
        if self.base != "http://127.0.0.1:8765" && self.base != "http://localhost:8765" {
            return Err(format!("Não foi possível acessar o serviço configurado em {}.", self.base));
        }
        let should_start = {
            let mut guard = owned_server().lock().map_err(|_| "Falha ao acessar o gerenciador do mVocalToolkit.".to_string())?;
            if self.health().is_ok() { return Ok(()); }
            if let Some(child) = guard.as_mut() {
                if child.try_wait().ok().flatten().is_some() { *guard = None; }
            }
            guard.is_none()
        };
        let log_path = std::env::temp_dir().join("copaiba-mvocaltoolkit.log");
        if should_start {
            progress("Preparando mVocalToolkit local…".into());
            let (program, args): (PathBuf, Vec<&str>) = if let Some(mvt) = find_command("mvt") {
                (mvt, vec!["serve", "--host", "127.0.0.1", "--port", "8765"])
            } else {
                let uv = match find_command("uv") {
                    Some(uv) => uv,
                    None => {
                        let progress_report = |message| progress(message);
                        install_uv(&progress_report)?
                    }
                };
                if cancel.load(Ordering::Relaxed) { return Err("Preparação do mVocalToolkit cancelada.".into()); }
                progress("Instalando o runtime do mVocalToolkit na primeira execução…".into());
                (uv, vec!["tool", "run", "--from", "git+https://github.com/Megageorgio/mVocalToolkit", "mvt", "serve", "--host", "127.0.0.1", "--port", "8765"])
            };
            if cancel.load(Ordering::Relaxed) { return Err("Preparação do mVocalToolkit cancelada.".into()); }
            let log = std::fs::File::create(&log_path).map_err(|error| format!("Não foi possível criar o log de inicialização: {error}"))?;
            let mut guard = owned_server().lock().map_err(|_| "Falha ao acessar o gerenciador do mVocalToolkit.".to_string())?;
            if self.health().is_ok() { return Ok(()); }
            if let Some(child) = guard.as_mut() {
                if child.try_wait().ok().flatten().is_some() { *guard = None; }
            }
            if guard.is_none() {
                let child = std::process::Command::new(program).args(args)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::from(log.try_clone().map_err(|error| format!("Não foi possível acompanhar o início do mVocalToolkit: {error}"))?))
                    .stderr(std::process::Stdio::from(log))
                    .spawn().map_err(|error| format!("Não foi possível iniciar mVocalToolkit: {error}"))?;
                *guard = Some(child);
            }
        }
        progress("Aguardando o serviço local do mVocalToolkit…".into());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
        let mut last_update = std::time::Instant::now();
        while std::time::Instant::now() < deadline {
            if cancel.load(Ordering::Relaxed) { return Err("Inicialização do mVocalToolkit cancelada.".into()); }
            if self.health().is_ok() { return Ok(()); }
            if let Ok(mut guard) = owned_server().lock() {
                if let Some(child) = guard.as_mut() {
                    if let Some(status) = child.try_wait().ok().flatten() {
                        *guard = None;
                        let details = std::fs::read_to_string(&log_path).unwrap_or_default();
                        let details = details.lines().rev().take(8).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
                        return Err(if details.is_empty() {
                            format!("mVocalToolkit encerrou durante a inicialização ({status}). Tente novamente.")
                        } else {
                            format!("mVocalToolkit encerrou durante a inicialização ({status}): {details}")
                        });
                    }
                }
            }
            if last_update.elapsed() >= std::time::Duration::from_secs(10) {
                let elapsed = 180 - deadline.saturating_duration_since(std::time::Instant::now()).as_secs();
                progress(format!("Iniciando mVocalToolkit local… ({elapsed} s)"));
                last_update = std::time::Instant::now();
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        let details = std::fs::read_to_string(log_path).unwrap_or_default();
        let details = details.lines().rev().take(8).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
        Err(if details.is_empty() {
            "O mVocalToolkit não respondeu após 3 minutos. Verifique a conexão com a internet e tente novamente.".into()
        } else {
            format!("O mVocalToolkit não respondeu após 3 minutos: {details}")
        })
    }

    fn stop_owned_service(&self) {
        let Ok(mut guard) = owned_server().lock() else { return; };
        if guard.is_some() && self.base == "http://127.0.0.1:8765" {
            let _ = self.http.post(format!("{}/shutdown", self.base)).send();
            if let Some(mut child) = guard.take() {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
                while std::time::Instant::now() < deadline && child.try_wait().ok().flatten().is_none() {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                if child.try_wait().ok().flatten().is_none() { let _ = child.kill(); }
            }
        }
    }

    fn languages(&self) -> Result<Vec<MvtLanguage>, String> {
        let value: serde_json::Value = self.http.get(format!("{}/languages?task=align", self.base)).send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|e| format!("Falha ao consultar idiomas de alinhamento: {e}"))?
            .json().map_err(|e| format!("Resposta inválida ao consultar idiomas: {e}"))?;
        let list = value.get("languages").and_then(|v| v.as_array())
            .ok_or_else(|| "A resposta do mVocalToolkit não contém a lista de idiomas.".to_string())?;
        Ok(list.iter().filter_map(|item| {
            let code = item.get("code")?.as_str()?.to_string();
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or(&code).to_string();
            let native_name = item.get("native_name").and_then(|v| v.as_str()).unwrap_or(&name).to_string();
            Some(MvtLanguage { code, name, native_name })
        }).collect())
    }

    fn models(&self, task: &str, language: Option<&str>) -> Result<Vec<MvtModel>, String> {
        let mut url = format!("{}/models?task={}", self.base, task);
        if let Some(language) = language { url.push_str(&format!("&language={}", language)); }
        let request = self.http.get(url);
        let value: serde_json::Value = request.send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|e| format!("Falha ao consultar modelos {task}: {e}"))?
            .json().map_err(|e| format!("Resposta inválida ao consultar modelos: {e}"))?;
        let list = value.as_array()
            .or_else(|| value.get("models").and_then(|v| v.as_array()))
            .or_else(|| value.get("items").and_then(|v| v.as_array()))
            .ok_or_else(|| "A resposta do mVocalToolkit não contém uma lista de modelos.".to_string())?;
        Ok(list.iter().filter_map(|item| {
            let id = item.get("id")?.as_str()?.to_string();
            Some(MvtModel {
                name: item.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string(),
                engine: item.get("engine").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                installed: item.get("installed").and_then(|v| v.as_bool()).unwrap_or(false),
                size_bytes: item.get("size_hint").and_then(|v| v.as_str()).and_then(parse_size_hint),
                id,
            })
        }).collect())
    }

    fn post_json(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
        self.http.post(format!("{}{}", self.base, path)).json(body).send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|e| format!("Falha em {path}: {e}"))?
            .json().map_err(|e| format!("Resposta inválida de {path}: {e}"))
    }

    fn run_job<F>(&self, path: &str, body: &serde_json::Value, cancel: &AtomicBool, progress: &F) -> Result<serde_json::Value, String>
    where F: Fn(String, Option<f32>) {
        let started = self.post_json(path, body)?;
        let job_id = started.get("id").and_then(|v| v.as_str())
            .ok_or_else(|| format!("O mVocalToolkit não retornou o ID do job de {path}."))?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2 * 60 * 60);
        loop {
            if cancel.load(Ordering::Relaxed) {
                let _ = self.http.post(format!("{}/jobs/{}/cancel", self.base, job_id)).send();
                return Err("Análise cancelada.".into());
            }
            if std::time::Instant::now() >= deadline { return Err("O job excedeu o limite de duas horas.".into()); }
            let state: serde_json::Value = self.http.get(format!("{}/jobs/{}?wait=10", self.base, job_id)).send()
                .and_then(reqwest::blocking::Response::error_for_status)
                .map_err(|e| format!("Falha ao acompanhar job {job_id}: {e}"))?
                .json().map_err(|e| format!("Estado inválido do job {job_id}: {e}"))?;
            let status = state.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
            let stage = state.get("stage").and_then(|v| v.as_str())
                .or_else(|| state.get("progress").and_then(|v| v.get("stage")).and_then(|v| v.as_str()))
                .unwrap_or(status);
            let detail = state.get("detail").or_else(|| state.get("progress").and_then(|v| v.get("detail")));
            let counts = detail.and_then(|v| Some((v.get("items_done")?.as_u64()?, v.get("items_total")?.as_u64()?)));
            let count_fraction = counts.and_then(|(done, total)| (total > 0 && done > 0).then_some(done as f32 / total as f32));
            let byte_fraction = detail.and_then(|v| Some((v.get("done")?.as_u64()?, v.get("total")?.as_u64()?)))
                .and_then(|(done, total)| (total > 0 && done > 0).then_some(done as f32 / total as f32));
            let reported_fraction = state.get("progress").and_then(|v| v.as_f64()).map(|v| v as f32).filter(|value| *value > 0.0);
            let fraction = reported_fraction.or(byte_fraction).or(count_fraction);
            let message = counts.map(|(done, total)| format!("mVocalToolkit: {stage} ({done}/{total})"))
                .unwrap_or_else(|| format!("mVocalToolkit: {stage}"));
            progress(message, fraction);
            match status {
                "done" => {
                    return Ok(state);
                }
                "failed" => {
                    let error = state.get("error").and_then(|v| v.as_str()).unwrap_or("erro sem detalhes");
                    return Err(format!("Job do mVocalToolkit falhou: {error}"));
                }
                "cancelled" => return Err("Job cancelado pelo mVocalToolkit.".into()),
                _ => {}
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_analysis<F>(settings: MvtSettings, items: Vec<MvtWorkItem>, cancel: Arc<AtomicBool>, progress: &F) -> Result<Vec<MvtOtoProposal>, String>
where F: Fn(String, Option<f32>) {
    let client = Client::new(&settings.endpoint)?;
    client.ensure_local_service(&cancel, |message| progress(message, None))?;
    client.health()?;
    let mut settings = settings;
    let models = client.models("align", Some(&settings.language))?;
    settings.align_model = models.iter().min_by_key(|model| (!model.installed, model.size_bytes.unwrap_or(u64::MAX)))
        .map(|model| model.id.clone())
        .ok_or_else(|| format!("Nenhum modelo de alinhamento disponível para {}.", settings.language))?;
    progress(format!("Modelo selecionado para {}: {}", settings.language, settings.align_model), None);
    let temp = tempfile::tempdir().map_err(|e| format!("Não foi possível criar pasta temporária: {e}"))?;
    let align_dir = temp.path().join("align");
    std::fs::create_dir_all(&align_dir).map_err(|e| format!("Falha criando saída temporária: {e}"))?;

    let mut seen = std::collections::HashSet::new();
    let mut accepted = Vec::new();
    let mut grouped_items: std::collections::HashMap<String, Vec<MvtWorkItem>> = std::collections::HashMap::new();
    let mut proposals = Vec::new();
    for item in items {
        let key = item.path.file_stem().and_then(|name| name.to_str()).unwrap_or("").to_lowercase();
        if !item.path.is_file() {
            if item.propose {
                proposals.push(MvtOtoProposal {
                    entry_index: item.entry_index, new_entry: false, filename: item.filename, alias: item.alias,
                    proposed_alias: None,
                    values: None, apply_fields: [false; 5], selected: false,
                    message: "Arquivo WAV não encontrado no caminho local.".into(),
                });
            }
        } else {
            if seen.insert(key.clone()) { accepted.push(item.clone()); }
            // VCV oto.ini files commonly have many aliases that point into the
            // same long recording. Align each recording once, then map labels
            // back to every selected entry using its own offset/preutterance.
            grouped_items.entry(key).or_default().push(item);
        }
    }
    if accepted.is_empty() { return Ok(proposals); }

    let api_items: Vec<_> = accepted.iter().map(|item| {
        let stem = item.path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let key = stem.to_lowercase();
        let related = grouped_items.get(&key).map(Vec::as_slice).unwrap_or(std::slice::from_ref(item));
        if related.iter().any(|entry| entry.generate_vcv_units) {
            let phones = japanese_phones_from_recording_alias(stem);
            serde_json::json!({"path": item.path.to_string_lossy(), "phonemes": phones})
        } else if settings.language.to_lowercase().starts_with("ja") {
            let phones = japanese_phones_for_recording(related);
            serde_json::json!({"path": item.path.to_string_lossy(), "phonemes": phones})
        } else {
            serde_json::json!({"path": item.path.to_string_lossy()})
        }
    }).collect();
    let mut align_body = serde_json::json!({
        "input": {"items": api_items},
        "model": settings.align_model,
        "language": settings.language,
        // The supplied text/phonemes are already grounded in oto aliases.
        // Do not run G2P over alignment tokens: phones such as `sh`, `t` and
        // `n` could otherwise be mistaken for unknown words.
        "g2p": "none",
        "output": {"formats": ["htk"], "dir": align_dir.to_string_lossy(), "layout": "beside", "overwrite": true}
    });
    // Japanese voicebanks already carry the intended phoneme sequence in their
    // aliases. Do not trigger mVocalToolkit's implicit Whisper transcription,
    // which downloads a separate multi-gigabyte model before alignment.
    if settings.language.to_lowercase().starts_with("ja") {
        align_body["transcribe"] = serde_json::Value::Null;
    }
    let align_job = client.run_job("/align", &align_body, &cancel, progress)?;
    let mut item_errors = std::collections::HashMap::new();
    let mut proposed_aliases = std::collections::HashMap::new();
    if let Some(results) = align_job.get("result").and_then(|v| v.get("items")).and_then(|v| v.as_array()) {
        for item in results {
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            let is_japanese = settings.language.to_lowercase().starts_with("ja");
            let tokens = item.get("tokens").and_then(|v| v.as_array()).map(|tokens| {
                let phones = tokens.iter().filter_map(|token| token.as_str().map(str::trim))
                    .filter(|token| !token.is_empty() && !matches!(token.to_lowercase().as_str(), "sp" | "sil" | "pau" | "ap" | "ep" | "br" | "h#"))
                    .collect::<Vec<_>>();
                if is_japanese { japanese_alias_units(&phones) } else { phones.join(" ") }
            }).unwrap_or_default();
            let text = item.get("text").and_then(|v| v.as_str()).unwrap_or("").trim();
            let transcript = if !tokens.trim().is_empty() { tokens } else { text.to_string() };
            if !name.is_empty() && !transcript.trim().is_empty() {
                let single_plain_item = grouped_items.get(&name).is_some_and(|entries| entries.len() == 1 && !entries[0].vcv_style);
                if single_plain_item {
                    let prefix = grouped_items.get(&name).and_then(|entries| entries.first())
                        .map(|work| alias_prefix(&work.alias)).unwrap_or("");
                    proposed_aliases.insert(name.clone(), format!("{prefix}{}", transcript.trim()));
                }
            }
            if item.get("ok").and_then(|v| v.as_bool()) == Some(false) {
                if let Some(error) = item.get("error").and_then(|v| v.as_str()) {
                    item_errors.insert(name, error.to_string());
                }
            }
        }
    }
    if let Some(errors) = align_job.get("item_errors").and_then(|v| v.as_array()) {
        for item in errors {
            if let (Some(name), Some(error)) = (item.get("item").and_then(|v| v.as_str()), item.get("error").and_then(|v| v.as_str())) {
                item_errors.insert(name.to_lowercase(), error.to_string());
            }
        }
    }

    let mut aligned: Vec<(MvtWorkItem, Vec<PhonemeSegment>)> = Vec::new();
    for item in accepted {
        let stem = item.path.file_stem().and_then(|name| name.to_str()).unwrap_or("").to_lowercase();
        if let Some(error) = item_errors.get(&stem) {
            for entry in grouped_items.remove(&stem).unwrap_or_else(|| vec![item.clone()]) {
                if !entry.propose { continue; }
                let alias = if entry.vcv_style { None } else { proposed_aliases.get(&stem).cloned() };
                proposals.push(failed_proposal_with_alias(entry, error, alias));
            }
            continue;
        }
        let path = lab_path(&align_dir, &item.path);
        match path.and_then(|p| parse_lab(&p)) {
            Ok(segments) if !segments.is_empty() => aligned.push((item, segments)),
            Ok(_) => for entry in grouped_items.remove(&stem).unwrap_or_else(|| vec![item.clone()]) {
                if !entry.propose { continue; }
                let alias = if entry.vcv_style { None } else { proposed_aliases.get(&stem).cloned() };
                proposals.push(failed_proposal_with_alias(entry, "O alinhador não retornou fonemas para este WAV.", alias));
            },
            Err(error) => for entry in grouped_items.remove(&stem).unwrap_or_else(|| vec![item.clone()]) {
                if !entry.propose { continue; }
                let alias = if entry.vcv_style { None } else { proposed_aliases.get(&stem).cloned() };
                proposals.push(failed_proposal_with_alias(entry, &error, alias));
            },
        }
    }

    let mut final_segments = Vec::new();
    if !settings.refine_model.trim().is_empty() && !aligned.is_empty() {
        let refine_dir = temp.path().join("refined");
        std::fs::create_dir_all(&refine_dir).map_err(|e| format!("Falha criando saída do refinador: {e}"))?;
        let refine_items: Vec<_> = aligned.iter().map(|(item, segments)| {
            let values: Vec<_> = segments.iter().map(|s| serde_json::json!([s.start_ms / 1000.0, s.end_ms / 1000.0, s.phone])).collect();
            serde_json::json!({"path": item.path.to_string_lossy(), "segments": values})
        }).collect();
        let body = serde_json::json!({
            "input": {"items": refine_items}, "model": settings.refine_model, "mode": "auto",
            "output": {"formats": ["htk"], "dir": refine_dir.to_string_lossy(), "layout": "beside", "overwrite": true}
        });
        let _ = client.run_job("/refine", &body, &cancel, progress)?;
        for (item, original) in aligned {
            let refined = lab_path(&refine_dir, &item.path).and_then(|p| parse_lab(&p));
            match refined {
                Ok(segments) if !segments.is_empty() => final_segments.push((item, segments, true)),
                Ok(_) => final_segments.push((item, original, false)),
                Err(_) => final_segments.push((item, original, false)),
            }
        }
    } else {
        final_segments = aligned.into_iter().map(|(item, segments)| (item, segments, false)).collect();
    }

    for (item, segments, refined) in final_segments {
        let stem = item.path.file_stem().and_then(|name| name.to_str()).unwrap_or("").to_lowercase();
        let entries = grouped_items.remove(&stem).unwrap_or_else(|| vec![item.clone()]);
        let wav = match crate::audio::load_wav(&item.path) {
            Ok(data) => data.wav,
            Err(error) => {
                for entry in entries { if entry.propose { proposals.push(failed_proposal(entry, &format!("Não foi possível ler o WAV: {error}"))); } }
                continue;
            }
        };
        let duration_ms = wav.duration_ms;
        for entry in entries {
            if !entry.propose { continue; }
            if entry.generate_vcv_units {
                let generated = match entry.scheme {
                    MvtOtoScheme::Cv => propose_cv_units(&segments, duration_ms, settings.margin_ms),
                    MvtOtoScheme::Vcv | MvtOtoScheme::Mixed => propose_vcv_units(&segments, duration_ms, settings.margin_ms),
                    MvtOtoScheme::Cvvc => propose_cvvc_units(&segments, duration_ms, settings.margin_ms),
                };
                match generated {
                    Some(units) => {
                        for (alias, values) in units {
                            proposals.push(MvtOtoProposal {
                                entry_index: usize::MAX,
                                new_entry: true,
                                filename: entry.filename.clone(),
                                alias: entry.alias.clone(),
                                proposed_alias: Some(alias),
                                values: Some(values),
                                apply_fields: [true; 5],
                                selected: true,
                                message: "Novas entradas sugeridas no formato escolhido. Revise aliases e marcadores antes de aplicar.".into(),
                            });
                        }
                    }
                    None => proposals.push(failed_proposal(entry, "Não foi possível separar as unidades do formato escolhido nesta gravação.")),
                }
                continue;
            }
            let (values, fields, message, proposed_alias) = if entry.vc_style {
                match propose_vc_entry(&segments, duration_ms, settings.margin_ms, &entry) {
                    Some((values, fields, alias)) => (
                        values, fields,
                        "Alinhamento da unidade VC concluído. Revise a transição e os marcadores antes de aplicar.".to_string(),
                        (entry.alias.trim().is_empty() || entry.alias.split_whitespace().collect::<Vec<_>>().join(" ") == alias).then_some(alias),
                    ),
                    None => (MvtOtoValues::default(), [false; 5], "Não foi possível localizar a transição vogal→consoante próxima aos marcadores atuais; tempos preservados.".to_string(), None),
                }
            } else if entry.vcv_style {
                match propose_vcv_entry(&segments, duration_ms, settings.margin_ms, &entry) {
                    Some((values, fields, alias)) => {
                        let current_alias = entry.alias.split_whitespace().collect::<Vec<_>>().join(" ");
                        let detected_alias = alias.split_whitespace().collect::<Vec<_>>().join(" ");
                        let alias_matches = current_alias.is_empty() || current_alias == detected_alias;
                        let message = if alias_matches {
                            "Alinhamento por entrada VCV concluído. Revise a vogal, o fonema e os marcadores antes de aplicar."
                        } else {
                            "O fonema detectado diverge do alias atual; alias preservado. Revise os marcadores e confirme o som manualmente."
                        };
                        (values, fields, message.to_string(), alias_matches.then_some(alias))
                    }
                    None => (MvtOtoValues::default(), [false; 5], "Não foi possível identificar a sílaba VCV perto dos marcadores atuais; alias e tempos preservados.".to_string(), None),
                }
            } else {
                let (values, fields, message) = propose_oto(&segments, duration_ms, settings.margin_ms, refined);
                let alias = proposed_aliases.remove(&stem);
                let message = if alias.is_some() { format!("{message} Revise o alias transcrito antes de aplicar.") } else { message };
                (values, fields, message, alias)
            };
            proposals.push(MvtOtoProposal {
                entry_index: entry.entry_index, new_entry: false, filename: entry.filename, alias: entry.alias,
                proposed_alias,
                values: fields.iter().any(|field| *field).then_some(values), apply_fields: fields, selected: true, message,
            });
        }
    }
    proposals.sort_by_key(|p| p.entry_index);
    Ok(proposals)
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_size_hint(value: &str) -> Option<u64> {
    let value = value.trim().to_lowercase();
    let split = value.find(|c: char| !c.is_ascii_digit() && c != '.' && c != ',')?;
    let amount = value[..split].replace(',', ".").parse::<f64>().ok()?;
    let unit = value[split..].trim();
    let multiplier = if unit.starts_with("gb") { 1_000_000_000.0 }
        else if unit.starts_with("mb") { 1_000_000.0 }
        else if unit.starts_with("kb") { 1_000.0 }
        else if unit.starts_with("tb") { 1_000_000_000_000.0 }
        else { return None; };
    (amount.is_finite() && amount >= 0.0).then_some((amount * multiplier) as u64)
}

#[cfg(not(target_arch = "wasm32"))]
fn failed_proposal(item: MvtWorkItem, message: &str) -> MvtOtoProposal {
    failed_proposal_with_alias(item, message, None)
}

#[cfg(not(target_arch = "wasm32"))]
fn failed_proposal_with_alias(item: MvtWorkItem, message: &str, proposed_alias: Option<String>) -> MvtOtoProposal {
    MvtOtoProposal {
        entry_index: item.entry_index, new_entry: false, filename: item.filename, alias: item.alias,
        selected: proposed_alias.is_some(), proposed_alias,
        values: None, apply_fields: [false; 5], message: message.into(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn propose_vcv_units(segments: &[PhonemeSegment], duration_ms: f64, margin_ms: f64) -> Option<Vec<(String, MvtOtoValues)>> {
    let mut units = Vec::new();
    for (index, vowel) in segments.iter().enumerate() {
        if is_silence(&vowel.phone) || !is_vowel(&vowel.phone) { continue; }
        let mut start = index;
        while start > 0 && !is_silence(&segments[start - 1].phone) && !is_vowel(&segments[start - 1].phone) {
            // Keep a moraic nasal as the VCV prefix when another consonant
            // follows it; otherwise `n ka` would be proposed as `nka`.
            if start < index && matches!(segments[start - 1].phone.trim().to_lowercase().as_str(), "n" | "m") { break; }
            start -= 1;
        }
        let cv = segments[start..=index].iter().map(|s| s.phone.as_str()).collect::<String>();
        let kana = crate::app::phonetic::convert_alias(&cv, true);
        if kana.is_empty() || kana.chars().any(|ch| ch.is_ascii_alphabetic()) { continue; }
        let prefix = start.checked_sub(1).map(|prefix_index| &segments[prefix_index])
            .filter(|segment| !is_silence(&segment.phone));
        let head = prefix.map(|segment| segment.phone.to_lowercase()).unwrap_or_else(|| "-".to_string());
        let consonant_start = segments[start].start_ms;
        let vowel_start = vowel.start_ms;
        let end = segments.iter().skip(index + 1)
            .find(|segment| !is_silence(&segment.phone) && !is_vowel(&segment.phone))
            .or_else(|| segments.iter().skip(index + 1).find(|segment| !is_silence(&segment.phone) && is_vowel(&segment.phone)))
            .map(|segment| segment.start_ms).unwrap_or_else(|| {
            segments.iter().rev().find(|s| !is_silence(&s.phone)).map(|s| s.end_ms).unwrap_or(duration_ms)
        });
        let values = propose_vcv_values(prefix, start < index, consonant_start, vowel_start, end, duration_ms, margin_ms);
        units.push((format!("{head} {kana}"), values));
    }
    (units.len() >= 2).then_some(units)
}

#[cfg(not(target_arch = "wasm32"))]
fn propose_vcv_values(
    prefix: Option<&PhonemeSegment>, has_consonant: bool, consonant_start: f64,
    vowel_start: f64, end: f64, duration_ms: f64, margin_ms: f64,
) -> MvtOtoValues {
    // A VCV sample must retain its prefix vowel for the crossfade. CV-style
    // offsetting at the consonant onset discards that transition entirely.
    let offset_anchor = prefix.map(|segment| segment.start_ms).unwrap_or(consonant_start);
    let offset = (offset_anchor - margin_ms).max(0.0).min(duration_ms);
    let preutter = (vowel_start - offset).max(0.0);
    let overlap = prefix.map(|segment| (segment.end_ms - offset).clamp(0.0, preutter)).unwrap_or(0.0);
    let cutoff_end = (end + margin_ms).min(duration_ms).max(offset);
    MvtOtoValues {
        offset,
        consonant: if has_consonant || prefix.is_some() {
            (preutter + 20.0).min((duration_ms - offset).max(0.0))
        } else { 0.0 },
        cutoff: (duration_ms - cutoff_end).max(0.0),
        preutter,
        overlap,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn propose_cv_units(segments: &[PhonemeSegment], duration_ms: f64, margin_ms: f64) -> Option<Vec<(String, MvtOtoValues)>> {
    let mut units = Vec::new();
    for (index, segment) in segments.iter().enumerate() {
        if is_silence(&segment.phone) || !is_vowel(&segment.phone) { continue; }
        let mut onset = index;
        while onset > 0 && !is_silence(&segments[onset - 1].phone) && !is_vowel(&segments[onset - 1].phone) { onset -= 1; }
        let phones = segments[onset..=index].iter().map(|s| s.phone.as_str()).collect::<String>();
        let kana = crate::app::phonetic::convert_alias(&phones, true);
        if kana.is_empty() || kana.chars().any(|ch| ch.is_ascii_alphabetic()) { continue; }
        let next = segments.iter().skip(index + 1).find(|s| !is_silence(&s.phone) && !is_vowel(&s.phone))
            .map(|s| s.start_ms).unwrap_or_else(|| segments.iter().rev().find(|s| !is_silence(&s.phone)).map(|s| s.end_ms).unwrap_or(duration_ms));
        let offset = (segments[onset].start_ms - margin_ms).max(0.0);
        let preutter = (segment.start_ms - offset).max(0.0);
        units.push((if onset == index { format!("- {kana}") } else { kana }, MvtOtoValues {
            offset, consonant: if onset == index { 0.0 } else { preutter + 20.0 },
            cutoff: (duration_ms - (next + margin_ms).min(duration_ms)).max(0.0),
            preutter, overlap: if onset == index { 0.0 } else { (preutter * 0.52).clamp(5.0, 140.0) },
        }));
    }
    (units.len() >= 2).then_some(units)
}

#[cfg(not(target_arch = "wasm32"))]
fn propose_cvvc_units(segments: &[PhonemeSegment], duration_ms: f64, margin_ms: f64) -> Option<Vec<(String, MvtOtoValues)>> {
    let mut units = Vec::new();
    let mut previous_vowel: Option<(String, usize)> = None;
    for (index, segment) in segments.iter().enumerate() {
        if is_silence(&segment.phone) || !is_vowel(&segment.phone) { continue; }
        let mut onset = index;
        while onset > 0 && !is_silence(&segments[onset - 1].phone) && !is_vowel(&segments[onset - 1].phone) { onset -= 1; }
        let cv_phones = segments[onset..=index].iter().map(|s| s.phone.as_str()).collect::<String>();
        let kana = crate::app::phonetic::convert_alias(&cv_phones, true);
        if kana.is_empty() || kana.chars().any(|ch| ch.is_ascii_alphabetic()) { continue; }
        if let Some((previous, previous_index)) = previous_vowel.take() {
            if onset < index {
                let consonant = segments[onset..index].iter().map(|s| s.phone.as_str()).collect::<String>();
                let offset = (segments[previous_index].start_ms - margin_ms).max(0.0);
                let preutter = (segments[onset].start_ms - offset).max(0.0);
                let tail = segments[index].start_ms;
                units.push((format!("{previous} {consonant}"), MvtOtoValues {
                    offset, consonant: preutter + 20.0, cutoff: (duration_ms - (tail + margin_ms).min(duration_ms)).max(0.0),
                    preutter, overlap: (preutter * 0.52).clamp(5.0, 140.0),
                }));
            }
        }
        let next = segments.iter().skip(index + 1).find(|s| !is_silence(&s.phone) && !is_vowel(&s.phone))
            .map(|s| s.start_ms).unwrap_or_else(|| segments.iter().rev().find(|s| !is_silence(&s.phone)).map(|s| s.end_ms).unwrap_or(duration_ms));
        let offset = (segments[onset].start_ms - margin_ms).max(0.0);
        let preutter = (segment.start_ms - offset).max(0.0);
        units.push((if onset == index { format!("- {kana}") } else { kana }, MvtOtoValues {
            offset, consonant: if onset == index { 0.0 } else { preutter + 20.0 },
            cutoff: (duration_ms - (next + margin_ms).min(duration_ms)).max(0.0), preutter,
            overlap: if onset == index { 0.0 } else { (preutter * 0.52).clamp(5.0, 140.0) },
        }));
        previous_vowel = Some((segment.phone.to_lowercase(), index));
    }
    (units.len() >= 2).then_some(units)
}

#[cfg(not(target_arch = "wasm32"))]
fn japanese_phones_from_recording_alias(stem: &str) -> Vec<String> {
    let mut phones = Vec::new();
    for syllable in stem.split('_').filter(|part| !part.is_empty()) {
        let lower = syllable.to_lowercase();
        if lower == "n" {
            phones.push("n".to_string());
            continue;
        }
        let Some(vowel) = lower.chars().last().filter(|ch| matches!(ch, 'a' | 'i' | 'u' | 'e' | 'o')) else { continue; };
        let onset = &lower[..lower.len() - vowel.len_utf8()];
        if onset.is_empty() {
            phones.push(vowel.to_string());
            continue;
        }
        let consonant = match onset {
            "sh" => "sh", "ch" => "ch", "ts" => "ts", "j" | "zh" => "j",
            "ky" => "ky", "sy" | "shy" => "sh", "ty" => "ch",
            "ny" => "ny", "hy" => "hy", "my" => "my", "ry" => "ry",
            "gy" => "gy", "by" => "by", "py" => "py", "dy" => "j",
            other => other,
        };
        phones.push(consonant.to_string());
        phones.push(vowel.to_string());
    }
    phones
}

#[cfg(not(target_arch = "wasm32"))]
fn japanese_phones_for_recording(entries: &[MvtWorkItem]) -> Vec<String> {
    let mut ordered = entries.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.offset_ms.total_cmp(&right.offset_ms)
        .then_with(|| left.entry_index.cmp(&right.entry_index)));
    ordered.into_iter().flat_map(|entry| {
        let alias = entry.alias.trim().trim_start_matches(['-', '_', ' ']);
        if alias.is_empty() { return Vec::new(); }
        let romaji = crate::app::phonetic::convert_alias(alias, false);
        let syllables = romaji.split_whitespace().collect::<Vec<_>>().join("_");
        japanese_phones_from_recording_alias(&syllables)
    }).collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn alias_prefix(alias: &str) -> &'static str {
    if alias.starts_with('_') { "_" }
    else if alias.starts_with('-') { "- " }
    else { "" }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn looks_like_vcv_alias(alias: &str) -> bool {
    let mut parts = alias.split_whitespace();
    let Some(head) = parts.next() else { return false; };
    let Some(tail) = parts.next() else { return false; };
    matches!(head.to_lowercase().as_str(), "a" | "i" | "u" | "e" | "o" | "n" | "あ" | "い" | "う" | "え" | "お")
        && !tail.is_empty()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn looks_like_vc_alias(alias: &str) -> bool {
    let mut parts = alias.split_whitespace();
    let Some(head) = parts.next() else { return false; };
    let Some(tail) = parts.next() else { return false; };
    if parts.next().is_some() || !matches!(head.to_lowercase().as_str(), "a" | "i" | "u" | "e" | "o" | "あ" | "い" | "う" | "え" | "お") { return false; }
    !tail.is_empty() && tail.chars().all(|ch| ch.is_ascii_alphabetic())
        && !tail.chars().any(|ch| matches!(ch.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))
}

#[cfg(not(target_arch = "wasm32"))]
fn propose_vcv_entry(
    segments: &[PhonemeSegment], duration_ms: f64, margin_ms: f64, item: &MvtWorkItem,
) -> Option<(MvtOtoValues, [bool; 5], String)> {
    let expected_vowel = item.offset_ms + item.preutter_ms;
    let vowel_index = segments.iter().enumerate()
        .filter(|(_, segment)| !is_silence(&segment.phone) && is_vowel(&segment.phone))
        .min_by(|(_, left), (_, right)| {
            (left.start_ms - expected_vowel).abs().total_cmp(&(right.start_ms - expected_vowel).abs())
        })
        .map(|(index, _)| index)?;

    // An alignment far from the entry's current vowel marker is not reliable
    // evidence for this oto row. In particular, do not reuse a nearby earlier
    // syllable for a later VCV entry when the expected region is missing.
    if (segments[vowel_index].start_ms - expected_vowel).abs() > 180.0 {
        return None;
    }

    let mut cv_start = vowel_index;
    let alias_head = item.alias.split_whitespace().next().unwrap_or_default().to_lowercase();
    let has_nasal_prefix = matches!(alias_head.as_str(), "n" | "ん");
    while cv_start > 0
        && !is_silence(&segments[cv_start - 1].phone)
        && !is_vowel(&segments[cv_start - 1].phone)
        // Japanese moraic n is the prefix mora in aliases like `n か`, not
        // part of the following CV onset `ka`.
        && !(has_nasal_prefix && matches!(segments[cv_start - 1].phone.trim().to_lowercase().as_str(), "n" | "m"))
    {
        cv_start -= 1;
    }
    let cv_phones = segments[cv_start..=vowel_index].iter().map(|segment| segment.phone.as_str()).collect::<String>();
    if cv_phones.is_empty() { return None; }
    let kana_cv = crate::app::phonetic::convert_alias(&cv_phones, true);
    if kana_cv.chars().any(|ch| ch.is_ascii_alphabetic()) { return None; }

    let prefix = if item.alias.trim_start().starts_with('-') {
        None
    } else {
        cv_start.checked_sub(1).map(|index| &segments[index])
            .filter(|segment| !is_silence(&segment.phone))
            .filter(|segment| is_vowel(&segment.phone) || matches!(segment.phone.trim().to_lowercase().as_str(), "n" | "m"))
    };
    let head = if item.alias.trim_start().starts_with('-') {
        "-".to_string()
    } else {
        match prefix.map(|segment| segment.phone.trim().to_lowercase()) {
            Some(phone) if is_vowel(&phone) => phone,
            Some(phone) if phone == "n" || phone == "m" => phone,
            _ => {
                let first = item.alias.split_whitespace().next()?;
                if matches!(first.to_lowercase().as_str(), "a" | "i" | "u" | "e" | "o" | "n") { first.to_lowercase() }
                else { return None; }
            }
        }
    };
    let proposed_alias = format!("{head} {kana_cv}");

    let consonant_start = segments[cv_start].start_ms;
    let vowel_start = segments[vowel_index].start_ms;
    let next_start = segments.iter().skip(vowel_index + 1)
        .find(|segment| !is_silence(&segment.phone) && !is_vowel(&segment.phone))
        .or_else(|| segments.iter().skip(vowel_index + 1).find(|segment| !is_silence(&segment.phone) && is_vowel(&segment.phone)))
        .map(|segment| segment.start_ms)
        .unwrap_or_else(|| segments.iter().rev().find(|segment| !is_silence(&segment.phone)).map(|segment| segment.end_ms).unwrap_or(duration_ms));
    let values = propose_vcv_values(prefix, cv_start < vowel_index, consonant_start, vowel_start, next_start, duration_ms, margin_ms);
    Some((values, [true; 5], proposed_alias))
}

#[cfg(not(target_arch = "wasm32"))]
fn propose_vc_entry(
    segments: &[PhonemeSegment], duration_ms: f64, margin_ms: f64, item: &MvtWorkItem,
) -> Option<(MvtOtoValues, [bool; 5], String)> {
    let expected = item.offset_ms + item.preutter_ms;
    let target = segments.windows(2).enumerate()
        .filter(|(_, pair)| is_vowel(&pair[0].phone) && !is_vowel(&pair[1].phone) && !is_silence(&pair[1].phone))
        .min_by(|(_, left), (_, right)| (left[1].start_ms - expected).abs().total_cmp(&(right[1].start_ms - expected).abs()))
        .map(|(index, _)| index)?;
    if (segments[target + 1].start_ms - expected).abs() > 180.0 { return None; }
    let vowel = &segments[target];
    let consonant = &segments[target + 1];
    let tail = item.alias.split_whitespace().nth(1)?.to_string();
    let alias = format!("{} {tail}", item.alias.split_whitespace().next()?);
    let offset = (vowel.start_ms - margin_ms).max(0.0).min(duration_ms);
    let preutter = (consonant.start_ms - offset).max(0.0);
    let next_vowel_end = segments.iter().skip(target + 2)
        .find(|segment| is_vowel(&segment.phone))
        .map(|segment| segment.start_ms)
        .unwrap_or(consonant.end_ms);
    let end = (next_vowel_end + margin_ms).min(duration_ms).max(offset);
    Some((MvtOtoValues {
        offset,
        consonant: (preutter + 20.0).min((duration_ms - offset).max(0.0)),
        cutoff: (duration_ms - end).max(0.0),
        preutter,
        overlap: (preutter * 0.52).clamp(5.0, 140.0).min(preutter + 20.0),
    }, [true; 5], alias))
}

#[cfg(not(target_arch = "wasm32"))]
fn japanese_alias_units(phones: &[&str]) -> String {
    let mut units = Vec::new();
    let mut current = String::new();
    for phone in phones {
        current.push_str(phone);
        if phone.chars().last().is_some_and(|ch| matches!(ch, 'a' | 'i' | 'u' | 'e' | 'o')) {
            units.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() { units.push(current); }
    units.join("_")
}

#[cfg(not(target_arch = "wasm32"))]
fn lab_path(dir: &std::path::Path, audio: &std::path::Path) -> Result<PathBuf, String> {
    let stem = audio.file_stem().ok_or_else(|| "Nome de WAV sem extensão válida.".to_string())?;
    let direct = dir.join(stem).with_extension("lab");
    let nested = dir.join("htk").join(stem).with_extension("lab");
    if direct.exists() { Ok(direct) }
    else if nested.exists() { Ok(nested) }
    else { Err(format!("O mVocalToolkit concluiu, mas não gerou o rótulo {}.lab.", stem.to_string_lossy())) }
}

#[derive(Clone, Debug)]
struct PhonemeSegment { start_ms: f64, end_ms: f64, phone: String }

#[cfg(not(target_arch = "wasm32"))]
fn parse_lab(path: &std::path::Path) -> Result<Vec<PhonemeSegment>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Não foi possível ler o alinhamento: {e}"))?;
    let mut raw = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 { continue; }
        let (Ok(start), Ok(end)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) else { continue; };
        if end >= start { raw.push((start, end, parts[2].trim_matches('"').to_string())); }
    }
    if raw.is_empty() { return Err("O arquivo de alinhamento não contém intervalos legíveis.".into()); }
    let max = raw.iter().map(|(_, end, _)| *end).fold(0.0_f64, f64::max);
    // HTK uses 100 ns ticks. Some exporters use seconds; accept both.
    let scale_to_ms = if max > 10_000.0 { 1.0 / 10_000.0 } else if max < 100.0 { 1000.0 } else { 1.0 };
    Ok(raw.into_iter().map(|(start, end, phone)| PhonemeSegment {
        start_ms: start * scale_to_ms, end_ms: end * scale_to_ms, phone,
    }).collect())
}

#[cfg(not(target_arch = "wasm32"))]
fn is_silence(phone: &str) -> bool {
    matches!(phone.trim().to_lowercase().as_str(), "sp" | "sil" | "pau" | "ap" | "ep" | "br" | "h#")
}

#[cfg(not(target_arch = "wasm32"))]
fn is_vowel(phone: &str) -> bool {
    let phone = phone.trim().to_lowercase();
    matches!(phone.as_str(), "a" | "e" | "i" | "o" | "u" | "aa" | "ae" | "ah" | "ao" | "aw" | "ax" | "ay" | "eh" | "er" | "ey" | "ih" | "iy" | "ow" | "oy" | "uh" | "uw" | "ɐ" | "ɑ" | "ɒ" | "æ" | "ə" | "ɚ" | "ɛ" | "ɜ" | "ɪ" | "ɔ" | "ʊ" | "ʌ" | "ɨ" | "ɯ" | "ɤ" | "ɵ" | "ɶ" | "ɞ")
}

#[cfg(not(target_arch = "wasm32"))]
fn propose_oto(segments: &[PhonemeSegment], duration_ms: f64, margin_ms: f64, refined: bool) -> (MvtOtoValues, [bool; 5], String) {
    let mut values = MvtOtoValues::default();
    let voiced: Vec<&PhonemeSegment> = segments.iter().filter(|s| !is_silence(&s.phone)).collect();
    if voiced.is_empty() {
        return (values, [false; 5], "Só foram encontrados rótulos de silêncio.".into());
    }
    let first = voiced.first().unwrap();
    let last = voiced.last().unwrap();
    values.offset = (first.start_ms - margin_ms).max(0.0).min(duration_ms);
    let trailing = (duration_ms - last.end_ms - margin_ms).max(0.0);
    values.cutoff = -trailing;

    // For CV/VCV labels, anchor preutterance at the latest consonant→vowel
    // boundary. This uses the model's alignment rather than an energy threshold.
    let boundary = segments.windows(2).rev().find(|pair| {
        !is_silence(&pair[0].phone) && is_vowel(&pair[1].phone) && !is_vowel(&pair[0].phone)
    }).map(|pair| pair[1].start_ms);
    let Some(vowel_start) = boundary else {
        return (values, [true, false, true, false, false], "Sem transição consoante→vogal identificável; somente offset e cutoff podem ser aplicados.".into());
    };
    values.preutter = (vowel_start - values.offset).max(0.0);
    values.consonant = (values.preutter + 20.0).min((duration_ms - values.offset).max(0.0));
    values.overlap = (values.preutter / 2.0).clamp(5.0, 30.0).min(values.consonant);
    let message = if refined {
        "Alinhamento e refinamento concluídos. Revise os marcadores na waveform antes de aplicar.".into()
    } else {
        "Alinhamento concluído. Valores de consonant/overlap incluem uma margem heurística; revise na waveform.".into()
    };
    (values, [true; 5], message)
}
