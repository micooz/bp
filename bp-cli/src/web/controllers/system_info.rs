use serde::Serialize;
use sysinfo::{RefreshKind, System, SystemExt};

use crate::web::common::{response::Response, state::State};

pub struct SystemInfoController;

impl SystemInfoController {
    pub async fn info(_req: tide::Request<State>) -> tide::Result {
        let specifics = RefreshKind::new().with_cpu().with_memory();
        let sys = System::new_with_specifics(specifics);

        #[derive(Debug, Serialize)]
        struct SystemInfo {
            system_name: Option<String>,
            system_hostname: Option<String>,
            system_kernel_version: Option<String>,
            system_os_version: Option<String>,
            uptime: u64,
            free_memory: u64,
            total_memory: u64,
            processors_count: usize,
            load_average: (f64, f64, f64),
        }

        let body = SystemInfo {
            system_name: sys.name(),
            system_hostname: sys.host_name(),
            system_kernel_version: sys.kernel_version(),
            system_os_version: sys.os_version(),
            uptime: sys.uptime(),
            free_memory: sys.free_memory(),
            total_memory: sys.total_memory(),
            processors_count: sys.processors().len(),
            load_average: (
                sys.load_average().one,
                sys.load_average().five,
                sys.load_average().fifteen,
            ),
        };

        let json = serde_json::to_string(&body)?;

        Response::success(serde_json::from_str(&json).unwrap())
    }
}
