use takehome::{field::GaloisField, graph_builder::*};
pub type Fp = GaloisField::<65537>;

// equivalent to the lambda x -> x/8 but written as a "vector-function". 
fn lambda_div8(val: Vec<Fp>) -> Fp {
    assert_eq!(val.len(), 1);
    val[0] / Fp::from(8)
}

#[test]
fn test_basic_function() {
    let mut builder = GraphBuilder::<Fp>::new();

    let x = builder.init();
    let x_squared = builder.mul(x.clone(), x.clone());
    let five = builder.constant(Fp::from(5));
    let x_squared_plus_5 = builder.add(x_squared.clone(), five.clone());
    let y = builder.add(x_squared_plus_5.clone(), x.clone());

    builder.set(&x, Fp::from(5));
    builder.fill_nodes();

    assert_eq!(x.read().value, 5);
    assert_eq!(x_squared.read().value, 25);
    assert_eq!(five.read().value, 5);
    assert_eq!(x_squared_plus_5.read().value, 30);
    assert_eq!(y.read().value, 35);
}

#[test]
fn test_multiple_access() {
    let mut builder = GraphBuilder::<Fp>::new();

    let x = builder.init();
    let y = builder.init();
    let z = builder.init();
    let w = builder.init(); 

    let x2 = builder.mul(x.clone(), x.clone());
    let xy = builder.mul(x.clone(), y.clone());
    let xz = builder.mul(x.clone(), z.clone());
    let xw = builder.mul(x.clone(), w.clone());

    builder.set(&x, Fp::from(5));
    builder.set(&y, Fp::from(5));
    builder.set(&z, Fp::from(45));
    builder.set(&w, Fp::from(6));

    builder.fill_nodes();
    assert_eq!(x.read(), Fp::from(5));
    assert_eq!(y.read(), Fp::from(5));
    assert_eq!(z.read(), Fp::from(45));
    assert_eq!(w.read(), Fp::from(6));

    assert_eq!(x2.read(), Fp::from(25));
    assert_eq!(xy.read(), Fp::from(25));
    assert_eq!(xz.read(), Fp::from(225));
    assert_eq!(xw.read(), Fp::from(30));
}

#[tokio::test]
async fn test_constraints() {
    let mut builder = GraphBuilder::<Fp>::new();
    let a = builder.init();
    let one = builder.constant(Fp::from(1)); 
    let eight = builder.constant(Fp::from(8));

    let b = builder.add(a.clone(), one); 

    let c = builder.init();
    let c_times_8 = builder.mul(c.clone(), eight.clone());

    builder.set(&a, Fp::from(13));
    builder.set(&c, Fp::from(2));

    builder.fill_nodes();
    builder.assert_equal(c_times_8.clone(), b.clone());

    let constraint_check = builder.check_constraints().await; 

    println!("{:?}", constraint_check); 
    println!("{:?}", c_times_8);
    println!("{:?}", b);
}

#[tokio::test]
async fn test_hints() {
    let mut builder = GraphBuilder::<Fp>::new();
    let a = builder.init();
    let one = builder.constant(Fp::from(1)); 
    let eight = builder.constant(Fp::from(8));

    let b = builder.add(a.clone(), one); 

    let c = builder.hint(&[b.clone()], lambda_div8);
    let c_times_8 = builder.mul(c.clone(), eight.clone());

    builder.set(&a, Fp::from(13));

    builder.fill_nodes();
    builder.assert_equal(c_times_8.clone(), b.clone());

    let constraint_check = builder.check_constraints().await; 

    println!("{:?}", constraint_check); 
    println!("{:?}", c_times_8);
    println!("{:?}", b);
}

#[tokio::test]
async fn test_lambda_gates() {
    let mut builder = GraphBuilder::<Fp>::new();

    let a = builder.init();
    let b = builder.init();

    let c = builder.mul(a.clone(), b.clone());

    fn lambda_div(params: Vec<Fp>) -> Fp {
        params[0] / params[1]
    }

    let d = builder.hint(&[c.clone(), b.clone()], lambda_div);

    builder.assert_equal(d.clone(), a.clone());

    builder.set(&a, Fp::from(234)); 
    builder.set(&b, Fp::from(123));

    builder.fill_nodes();
    let passed_constraints = builder.check_constraints().await; 

    assert!(passed_constraints);
    assert_eq!(a.read().value, 234);
    assert_eq!(b.read().value, 123);
    assert_eq!(c.read().value, 28782);
    assert_eq!(d.read().value, 234);
}