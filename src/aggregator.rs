use std::sync::Arc;
use std::time::Duration;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use tokio::sync::mpsc;

use crate::models::{
    SubtitleDownloadRequest, SubtitleDownloadResponse, SubtitleSearchRequest, SubtitleSearchResult,
};
use crate::providers::SubtitleProvider;
use crate::providers::{
    addic7ed::Addic7edProvider, animekalesi::AnimekalesiProvider,
    animesubinfo::AnimesubinfoProvider, animetosho::AnimeToshoProvider, assrt::AssrtProvider,
    betaseries::BetaSeriesProvider, bsplayer::BsPlayerProvider, gestdown::GestdownProvider,
    greeksubs::GreekSubsProvider, greeksubtitles::GreekSubtitlesProvider,
    hosszupuska::HosszupuskaProvider, jimaku::JimakuProvider, ktuvit::KtuvitProvider,
    legendasdivx::LegendasDivxProvider, legendasnet::LegendasNetProvider,
    napiprojekt::NapiprojektProvider, napisy24::Napisy24Provider, nekur::NekurProvider,
    opensubtitles::OpenSubtitlesProvider, podnapisi::PodnapisiProvider,
    regielive::RegieLiveProvider, shooter::ShooterProvider, soustitreseu::SoustitreseuProvider,
    subdl::SubdlProvider, subf2m::Subf2mProvider, subs4free::Subs4FreeProvider,
    subs4series::Subs4SeriesProvider, subscenter::SubsCenterProvider, subsource::SubSourceProvider,
    subsro::SubsRoProvider, subssabbz::SubssabbzProvider, subsunacs::SubsunacsProvider,
    subsynchro::SubsynchroProvider, subtis::SubtisProvider, subtitrarinoi::SubtitrariNoiProvider,
    subtitriid::SubtitriIdProvider, subtitulamostv::SubtitulamosTvProvider, subx::SubxProvider,
    supersubtitles::SuperSubtitlesProvider, thesubdb::TheSubDbProvider, titlovi::TitloviProvider,
    titrari::TitrariProvider, titulky::TitulkyProvider, turkcealtyazi::TurkcealtyaziProvider,
    tvsubtitles::TvSubtitlesProvider, wizdom::WizdomProvider, xsubs::XSubsProvider,
    xunlei::XunleiSubtitleProvider, yavkanet::YavkanetProvider, yify::YifyProvider,
    zimuku::ZimukuProvider,
};

/// Per-provider search timeout. Prevents a slow provider from blocking all results.
const PROVIDER_TIMEOUT: Duration = Duration::from_secs(30);

pub struct SubtitleAggregator {
    providers: Vec<Arc<dyn SubtitleProvider>>,
}

impl SubtitleAggregator {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Build an aggregator with all supported providers registered.
    /// `staging` is the base directory for provider-specific cache/staging files.
    pub fn with_all_providers(staging: &str) -> Self {
        let mut agg = Self::new();

        // ── Chinese / Asian ──
        agg.add_provider(Arc::new(AssrtProvider::new(staging)));
        agg.add_provider(Arc::new(ZimukuProvider::new(staging)));
        agg.add_provider(Arc::new(AnimekalesiProvider::new()));
        agg.add_provider(Arc::new(AnimesubinfoProvider::new()));
        agg.add_provider(Arc::new(AnimeToshoProvider::new()));

        // ── English / multi-language ──
        agg.add_provider(Arc::new(PodnapisiProvider::new()));
        agg.add_provider(Arc::new(SubdlProvider::new(
            std::env::var("SUBDL_API_KEY").ok(),
        )));
        agg.add_provider(Arc::new(Subf2mProvider::new(staging)));
        agg.add_provider(Arc::new(YifyProvider::new(staging)));
        agg.add_provider(Arc::new(TvSubtitlesProvider::new()));
        agg.add_provider(Arc::new(BsPlayerProvider::new()));
        agg.add_provider(Arc::new(NapiprojektProvider::new()));
        agg.add_provider(Arc::new(Napisy24Provider::new()));
        agg.add_provider(Arc::new(NekurProvider::new()));
        agg.add_provider(Arc::new(SubtitriIdProvider::new()));
        agg.add_provider(Arc::new(SuperSubtitlesProvider::new()));
        agg.add_provider(Arc::new(HosszupuskaProvider::new()));
        agg.add_provider(Arc::new(GestdownProvider::new()));
        agg.add_provider(Arc::new(SubtisProvider::new()));
        agg.add_provider(Arc::new(SubtitulamosTvProvider::new()));

        // ── French ──
        agg.add_provider(Arc::new(SoustitreseuProvider::new(staging)));
        agg.add_provider(Arc::new(SubsynchroProvider::new(staging)));

        // ── Portuguese ──
        agg.add_provider(Arc::new(LegendasDivxProvider::new(staging)));
        agg.add_provider(Arc::new(LegendasNetProvider::new(staging)));

        // ── Greek ──
        agg.add_provider(Arc::new(GreekSubsProvider::new(staging)));
        agg.add_provider(Arc::new(GreekSubtitlesProvider::new(staging)));
        agg.add_provider(Arc::new(Subs4FreeProvider::new(staging)));
        agg.add_provider(Arc::new(Subs4SeriesProvider::new(staging)));
        agg.add_provider(Arc::new(XSubsProvider::new(staging)));
        agg.add_provider(Arc::new(SubsCenterProvider::new(staging)));

        // ── Hebrew ──
        agg.add_provider(Arc::new(WizdomProvider::new(staging)));

        // ── Romanian ──
        agg.add_provider(Arc::new(TitrariProvider::new(staging)));
        agg.add_provider(Arc::new(RegieLiveProvider::new(staging)));
        agg.add_provider(Arc::new(SubtitrariNoiProvider::new(staging)));
        agg.add_provider(Arc::new(SubsRoProvider::new(staging)));

        // ── Turkish ──
        agg.add_provider(Arc::new(TurkcealtyaziProvider::new()));

        // ── Bulgarian ──
        agg.add_provider(Arc::new(SubssabbzProvider::new()));
        agg.add_provider(Arc::new(SubsunacsProvider::new()));
        agg.add_provider(Arc::new(YavkanetProvider::new()));

        // ── Optional credential providers ──
        agg.add_provider(Arc::new(Addic7edProvider::new(
            std::env::var("ADDIC7ED_USER").ok(),
            std::env::var("ADDIC7ED_PASS").ok(),
        )));
        agg.add_provider(Arc::new(SubSourceProvider::new(
            std::env::var("SUBSOURCE_API_KEY").ok(),
        )));
        agg.add_provider(Arc::new(SubxProvider::new()));
        agg.add_provider(Arc::new(BetaSeriesProvider::new(staging)));
        agg.add_provider(Arc::new(KtuvitProvider::new(staging)));

        if let Ok(api_key) = std::env::var("JIMAKU_API_KEY")
            && !api_key.is_empty()
        {
            agg.add_provider(Arc::new(JimakuProvider::new(api_key)));
        }

        if let (Ok(user), Ok(pass)) = (std::env::var("TITLOVI_USER"), std::env::var("TITLOVI_PASS"))
        {
            agg.add_provider(Arc::new(TitloviProvider::new(user, pass, staging)));
        }

        if let (Ok(user), Ok(pass)) = (std::env::var("TITULKY_USER"), std::env::var("TITULKY_PASS"))
        {
            agg.add_provider(Arc::new(TitulkyProvider::new(user, pass, staging)));
        }

        if let Ok(api_key) = std::env::var("OPENSUBTITLES_API_KEY")
            && !api_key.is_empty()
        {
            agg.add_provider(Arc::new(OpenSubtitlesProvider::new(api_key)));
        }

        // ── Hash-based providers (skip silently if no file_hash) ──
        agg.add_provider(Arc::new(ShooterProvider::new()));
        agg.add_provider(Arc::new(XunleiSubtitleProvider::new()));
        agg.add_provider(Arc::new(TheSubDbProvider::new()));

        agg
    }

    pub fn add_provider(&mut self, provider: Arc<dyn SubtitleProvider>) {
        self.providers.push(provider);
    }

    /// Concurrently search all providers, merge results.
    /// Each provider is given PROVIDER_TIMEOUT seconds before being cancelled.
    pub async fn search(
        &self,
        request: &SubtitleSearchRequest,
    ) -> Result<Vec<SubtitleSearchResult>, String> {
        // Wrap in Arc so each spawned task increments a ref-count instead of cloning all strings.
        let request = Arc::new(request.clone());
        let mut handles = Vec::with_capacity(self.providers.len());

        for provider in &self.providers {
            let provider = Arc::clone(provider);
            let request = Arc::clone(&request);
            let handle = tokio::spawn(async move {
                let fut = provider.search(&request);
                match tokio::time::timeout(PROVIDER_TIMEOUT, fut).await {
                    Ok(Ok(results)) => results,
                    Ok(Err(_)) | Err(_) => Vec::new(),
                }
            });
            handles.push(handle);
        }

        let mut all_results = Vec::new();
        for handle in handles {
            if let Ok(results) = handle.await {
                all_results.extend(results);
            }
        }

        Ok(all_results)
    }

    /// Streaming search: sends each provider's results as soon as they arrive.
    /// The sender is closed automatically when all providers complete.
    pub async fn search_streaming(
        &self,
        request: SubtitleSearchRequest,
        tx: mpsc::Sender<Vec<SubtitleSearchResult>>,
    ) {
        let request = Arc::new(request);
        let mut handles = Vec::with_capacity(self.providers.len());

        for provider in &self.providers {
            let provider = Arc::clone(provider);
            let request = Arc::clone(&request);
            let tx = tx.clone();
            handles.push(tokio::spawn(async move {
                match tokio::time::timeout(PROVIDER_TIMEOUT, provider.search(&request)).await {
                    Ok(Ok(results)) if !results.is_empty() => {
                        let _ = tx.send(results).await;
                    }
                    _ => {}
                }
                // tx clone drops here; channel closes when last clone drops
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
        // original tx drops here, closing the channel → stream ends
    }

    /// Download from the specific provider
    pub async fn download(
        &self,
        request: &SubtitleDownloadRequest,
    ) -> Result<SubtitleDownloadResponse, String> {
        let provider = self
            .providers
            .iter()
            .find(|p| p.name() == request.provider)
            .ok_or_else(|| format!("未找到 provider: {}", request.provider))?;

        let downloaded = provider.download(request).await?;

        Ok(SubtitleDownloadResponse {
            name: downloaded.name,
            format: downloaded.format,
            content_base64: BASE64.encode(&downloaded.content),
        })
    }

    /// List available provider names
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.iter().map(|p| p.name()).collect()
    }
}

impl Default for SubtitleAggregator {
    fn default() -> Self {
        Self::new()
    }
}
