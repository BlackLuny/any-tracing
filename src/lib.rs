use opentelemetry::Context;
use std::{collections::HashMap, hash::Hash};
use tracing::{info_span, span};
// use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use typedmap::{TypedMap, TypedMapKey};

#[derive(Hash, Eq, PartialEq)]
struct AnyKey<T> {
    val: T,
}

impl<T> From<(T,)> for AnyKey<(T,)> {
    fn from(s: (T,)) -> Self {
        AnyKey { val: s }
    }
}

impl<T0, T1> From<(T0, T1)> for AnyKey<(T0, T1)> {
    fn from(s: (T0, T1)) -> Self {
        AnyKey { val: s }
    }
}

impl<T0, T1, T2> From<(T0, T1, T2)> for AnyKey<(T0, T1, T2)> {
    fn from(s: (T0, T1, T2)) -> Self {
        AnyKey { val: s }
    }
}

impl<T0, T1, T2, T3> From<(T0, T1, T2, T3)> for AnyKey<(T0, T1, T2, T3)> {
    fn from(s: (T0, T1, T2, T3)) -> Self {
        AnyKey { val: s }
    }
}

impl<T: Hash + Eq + ToKeys> TypedMapKey for AnyKey<T> {
    type Value = (HashMap<String, String>, Span);
}

trait ToKeys {
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
fn enter1<T1: 'static + Eq + Hash + ToKeys + Clone>(
    map: &mut TypedMap,
    k1: AnyKey<T1>,
) -> (Context, &mut Span) {
    use opentelemetry::global::get_text_map_propagator;
    let v = k1.val.clone();
    let (m, s) = map.entry(k1).or_insert_with(|| {
        let mut m = HashMap::new();
        let s = format!("span:{}", v.to_keys());
        let cur = info_span!("span", s = s.as_str());
        get_text_map_propagator(|p| {
            p.inject_context(&cur.context(), &mut m);
        });
        (m, cur)
    });
    let ctx = get_text_map_propagator(|p| p.extract(m));
    (ctx, s)
}

macro_rules! enter_parrent {
    ($map: expr, $t0: expr, $t1: expr, $t2: expr, $t3: expr) => {{
        let (p0, _) = enter1($map, ($t0,).into());
        let _g0 = p0.attach();
        let (p1, _) = enter1($map, ($t0, $t1).into());
        let _g1 = p1.attach();
        let (p2, _) = enter1($map, ($t0, $t1, $t2).into());
        let _g2 = p2.attach();
        let (_, s) = enter1($map, ($t0, $t1, $t2, $t3).into());
        s
    }};
    ($map: expr, $t0: expr, $t1: expr, $t2: expr) => {{
        let (p0, _) = enter1($map, ($t0,).into());
        let _g0 = p0.attach();
        let (p1, _) = enter1($map, ($t0, $t1).into());
        let _g1 = p1.attach();
        let (_, s) = enter1($map, ($t0, $t1, $t2).into());
        s
    }};
    ($map: expr, $t0: expr, $t1: expr) => {{
        let (p0, _) = enter1($map, ($t0,).into());
        let _g0 = p0.attach();
        let (_, s) = enter1($map, ($t0, $t1).into());
        s
    }};
    ($map: expr, $t0: expr) => {{
        let (_, s) = enter1($map, ($t0,).into());
        s
    }};
}
#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use opentelemetry::{
        global::{set_text_map_propagator, shutdown_tracer_provider},
        runtime::Tokio,
        sdk::propagation::TraceContextPropagator,
        trace::TraceContextExt,
        Context,
    };
    use tracing::{collect::set_global_default, info, info_span, span, Span};
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    use tracing_subscriber::{subscribe::CollectExt, Registry};
    use typedmap::TypedMap;

    use crate::enter1;

    #[tokio::main]
    #[test]
    async fn it_works() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut map = TypedMap::new();

        // enter1(&mut map, AnyKey{val:(1,)});
        // enter3(&mut map, 1.into(), 2.into(), 3.into());
        // enter2(&mut map, 1.into(), 2.into());
        // enter1(&mut map, 1.into());
        set_text_map_propagator(TraceContextPropagator::new());

        let tracer = opentelemetry_jaeger::new_collector_pipeline()
            .with_reqwest()
            .with_service_name("trace_demo")
            .with_endpoint("http://127.0.0.1:14268/api/traces")
            .install_batch(Tokio)?;
        let telemetry = tracing_opentelemetry::subscriber().with_tracer(tracer);
        let collector = Registry::default().with(telemetry);

        set_global_default(collector)?;
        for i in 0 as i32..1 {
            for j in 0 as i32..3 {
                for k in 0 as i32..3 {
                    for m in 0 as i32..1 {
                        let sp = enter_parrent!(&mut map, i, j, k, m);
                        let e = sp.enter();
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
                    let sp = enter_parrent!(&mut map, i, j, k);
                    let e = sp.enter();
                    let sub_span = info_span!("inner");
                    let _g = sub_span.enter();
                    info!("abc");
                }
            }
        }

        for i in 0 as i32..1 {
            for j in 0 as i32..300 {
                let sp = enter_parrent!(&mut map, i, j);
                let e = sp.enter();
                let sub_span = info_span!("inner");
                let _g = sub_span.enter();
                info!("abc");
            }
        }

        for i in 0 as i32..300 {
            let k = format!("task: {i}");
            let sp = enter_parrent!(&mut map, k);
            let e = sp.enter();
            let sub_span = info_span!("inner");
            let _g = sub_span.enter();
            info!("abc");
        }
        drop(map);
        shutdown_tracer_provider();
        Ok(())
    }
}
