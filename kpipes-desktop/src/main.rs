use crate::flow::Flow;
use kpipes_core::KPipes;

mod convert;
mod flow;

fn main() {
    env_logger::init();

    let mut flow = Flow::new(KPipes::init);
    flow.event(KPipes::event);
    flow.update(KPipes::update);
    flow.render(KPipes::render);
    flow.title = "KPipes".to_string();
    flow.fullscreen = true;

    flow.start().unwrap();
}
