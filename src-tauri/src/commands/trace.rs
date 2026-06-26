use crate::state::{AppState, BackendId, BackendState};
use crate::tracer::{RasterImage, TraceOptions};

#[tauri::command]
pub async fn trace_image(
    state: tauri::State<'_, AppState>,
    file_path: String,
    opts: TraceOptions,
) -> Result<String, String> {
    let bytes = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let dyn_img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let rgba = dyn_img.to_rgba8();
    let raster = RasterImage {
        data: rgba.as_raw().clone(),
        width: rgba.width(),
        height: rgba.height(),
    };
    let tracer = state.tracer.lock().unwrap();
    tracer
        .trace(&raster, &opts)
        .map(|o| o.svg)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_backend(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    backend: BackendId,
) -> Result<(), String> {
    use crate::tracer::live::LiveBackend;
    use crate::tracer::starvector::{StarVectorBackend, StarVectorTier};
    use crate::tracer::vtracer::VtracerBackend;

    {
        let statuses = state.backend_statuses.lock().unwrap();
        if let Some(s) = statuses.iter().find(|s| s.id == backend) {
            if !matches!(s.state, BackendState::Ready) {
                return Err(format!(
                    "{:?} is not ready — install it via Settings → Enhanced Backends",
                    backend
                ));
            }
        }
    }

    let new_tracer: Box<dyn crate::tracer::ImageTracer> = match backend {
        BackendId::Vtracer => Box::new(VtracerBackend),
        BackendId::Live => Box::new(LiveBackend { app: app.clone() }),
        BackendId::StarVector1B => {
            Box::new(StarVectorBackend { app: app.clone(), tier: StarVectorTier::OneB })
        }
        BackendId::StarVector8B => {
            Box::new(StarVectorBackend { app, tier: StarVectorTier::EightB })
        }
    };
    *state.tracer.lock().unwrap() = new_tracer;
    Ok(())
}
