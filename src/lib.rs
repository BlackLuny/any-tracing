use opentelemetry::Context;
use std::{collections::HashMap};

pub use tracing_opentelemetry::OpenTelemetrySpanExt;
pub use opentelemetry::global::get_text_map_propagator;
pub use opentelemetry::propagation::TextMapPropagator;
pub use tracing::{info_span};
pub use tracing::Span;
pub trait ToKeys {
    fn to_keys(&self) -> String;
}

impl<T0: ToString, T1: ToString, T2: ToString, T3: ToString> ToKeys for (T0, T1, T2, T3) {
    fn to_keys(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.0.to_string(),
            self.1.to_string(),
            self.2.to_string(),
            self.3.to_string()
        )
    }
}

impl<T0: ToString, T1: ToString, T2: ToString> ToKeys for (T0, T1, T2) {
    fn to_keys(&self) -> String {
        format!(
            "{}-{}-{}",
            self.0.to_string(),
            self.1.to_string(),
            self.2.to_string()
        )
    }
}
impl<T0: ToString, T1: ToString> ToKeys for (T0, T1) {
    fn to_keys(&self) -> String {
        format!("{}-{}", self.0.to_string(), self.1.to_string())
    }
}

impl<T0: ToString> ToKeys for (T0,) {
    fn to_keys(&self) -> String {
        self.0.to_string()
    }
}
pub fn enter1(
    map: &mut HashMap<String, (HashMap<String, String>, Span)>,
    k1: String,
) -> (Context, &mut Span) {
    let s_name = format!("span:{}", k1);
    let (m, s) = map.entry(k1).or_insert_with(|| {
        let mut m = HashMap::new();
        let cur = info_span!("span", s_name = s_name.as_str());
        get_text_map_propagator(|p| {
            p.inject_context(&cur.context(), &mut m);
        });
        (m, cur)
    });
    let ctx = get_text_map_propagator(|p| p.extract(m));
    (ctx, s)
}

#[macro_export]
macro_rules! enter_parrent {
    ($map: expr, $t0: expr, $t1: expr, $t2: expr, $t3: expr) => {{
        use $crate::ToKeys;
        use $crate::enter1;
        let (p0, _) = enter1($map, ($t0,).to_keys());
        let _g0 = p0.attach();
        let (p1, _) = enter1($map, ($t0, $t1).to_keys());
        let _g1 = p1.attach();
        let (p2, _) = enter1($map, ($t0, $t1, $t2).to_keys());
        let _g2 = p2.attach();
        let (_, s) = enter1($map, ($t0, $t1, $t2, $t3).to_keys());
        s
    }};
    ($map: expr, $t0: expr, $t1: expr, $t2: expr) => {{
        use $crate::ToKeys;
        use $crate::enter1;
        let (p0, _) = enter1($map, ($t0,).to_keys());
        let _g0 = p0.attach();
        let (p1, _) = enter1($map, ($t0, $t1).to_keys());
        let _g1 = p1.attach();
        let (_, s) = enter1($map, ($t0, $t1, $t2).to_keys());
        s
    }};
    ($map: expr, $t0: expr, $t1: expr) => {{
        use $crate::ToKeys;
        use $crate::enter1;
        let (p0, _) = enter1($map, ($t0,).to_keys());
        let _g0 = p0.attach();
        let (_, s) = enter1($map, ($t0, $t1).to_keys());
        s
    }};
    ($map: expr, $t0: expr) => {{
        use $crate::ToKeys;
        use $crate::enter1;
        let (_, s) = enter1($map, ($t0,).to_keys());
        s
    }};
}


#[cfg(test)]
mod tests {

    use std::{collections::HashMap, sync::{RwLock, Arc}};

    use opentelemetry::{
        global::{set_text_map_propagator, shutdown_tracer_provider},
        runtime::Tokio,
        sdk::propagation::TraceContextPropagator,
    };
    use tracing::{collect::set_global_default, info, info_span};
    use tracing_subscriber::{subscribe::CollectExt, Registry};

    #[tokio::main]
    #[test]
    async fn it_works() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
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
}
