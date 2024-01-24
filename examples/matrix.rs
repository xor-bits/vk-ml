use vk_ml::Tensor;

//

fn main() {
    tracing_subscriber::fmt::init();

    let a = Tensor::<f32>::empty([3, 4]);
    let b = Tensor::<f32>::empty([3, 4]);

    a.dbg();
    b.dbg();
}
