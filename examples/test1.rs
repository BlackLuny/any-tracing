use std::{sync::{Arc, RwLock}, collections::HashMap};

use any_tracing::info_span;
use opentelemetry::{global::{set_text_map_propagator, shutdown_tracer_provider}, runtime::Tokio, sdk::propagation::TraceContextPropagator};
use tracing::{collect::set_global_default, info};
use tracing_subscriber::{Registry, subscribe::CollectExt};

#[macro_use]
extern crate any_tracing;
#[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let map = Arc::new(RwLock::new(HashMap::new()));
        set_text_map_propagator(TraceContextPropagator::new());

        let tracer = opentelemetry_jaeger::new_collector_pipeline()
            .with_reqwest()
            .with_service_name("trace_demo")
            .with_endpoint("http://127.0.0.1:14268/api/traces")
            .install_batch(Tokio)?;
        let telemetry = tracing_opentelemetry::subscriber().with_tracer(tracer);
        let collector = Registry::default().with(telemetry);

        set_global_default(collector)?;
        let mut lock = map.write().unwrap();
        for i in 0 as i32..1 {
            for j in 0 as i32..3 {
                for k in 0 as i32..3 {
                    for m in 0 as i32..1 {
                        let sp = enter_parrent!(&mut lock, i, j, k, m);
                        let _e = sp.enter();
                        let sub_span = info_span!("inner");
                        let _g = sub_span.enter();
                        info!("abc");
                    }
                }
            }
        }
        for i in 0 as i32..1 {
            for j in 0 as i32..3 {
                for k in 0 as i32..300 {
                    let sp = enter_parrent!(&mut lock, i, j, k);
                    let _e = sp.enter();
                    let sub_span = info_span!("inner");
                    let _g = sub_span.enter();
                    info!("abc");
                }
            }
        }

        for i in 0 as i32..1 {
            for j in 0 as i32..300 {
                let sp = enter_parrent!(&mut lock, i, j);
                let _e = sp.enter();
                let sub_span = info_span!("inner");
                let _g = sub_span.enter();
                info!("abc");
            }
        }

        for i in 0 as i32..300 {
            let k = format!("task: {i}");
            let sp = enter_parrent!(&mut lock, k);
            let _e = sp.enter();
            let sub_span = info_span!("inner");
            let _g = sub_span.enter();
            info!("abc");
        }
        drop(lock);
        drop(map);
        shutdown_tracer_provider();
        Ok(())
    }