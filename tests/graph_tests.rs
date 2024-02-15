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
    let x_squared = builder.mul(&x, &x);
    
    let five = builder.constant(Fp::from(5));
    let x_squared_plus_5 = builder.add(&x_squared, &five);
    let y = builder.add(&x_squared_plus_5, &x);

    builder.set(&x, Fp::from(5));

    builder.fill_nodes();

    assert_eq!(x.read().unwrap().value.unwrap().value, 5);
    assert_eq!(x_squared.read().unwrap().value.unwrap().value, 25);
    assert_eq!(five.read().unwrap().value.unwrap().value, 5);
    assert_eq!(x_squared_plus_5.read().unwrap().value.unwrap().value, 30);
    assert_eq!(y.read().unwrap().value.unwrap().value, 35);
}

#[test]
fn test_multiple_access() {
    let mut builder = GraphBuilder::<Fp>::new();

    let x = builder.init();
    let y = builder.init();
    let z = builder.init();
    let w = builder.init(); 

    let x2 = builder.mul(&x, &x);
    let xy = builder.mul(&x, &y);
    let xz = builder.mul(&x, &z);
    let xw = builder.mul(&x, &w);

    builder.set(&x, Fp::from(5));
    builder.set(&y, Fp::from(5));
    builder.set(&z, Fp::from(45));
    builder.set(&w, Fp::from(6));

    builder.fill_nodes();
    assert_eq!(x.read().unwrap().value.unwrap().value, 5);
    assert_eq!(y.read().unwrap().value.unwrap().value, 5);
    assert_eq!(z.read().unwrap().value.unwrap().value, 45);
    assert_eq!(w.read().unwrap().value.unwrap().value, 6);

    assert_eq!(x2.read().unwrap().value.unwrap().value, 25);
    assert_eq!(xy.read().unwrap().value.unwrap().value, 25);
    assert_eq!(xz.read().unwrap().value.unwrap().value, 225);
    assert_eq!(xw.read().unwrap().value.unwrap().value, 30);
}

#[tokio::test]
async fn test_constraints() {
    let mut builder = GraphBuilder::<Fp>::new();
    let a = builder.init();
    let one = builder.constant(Fp::from(1)); 
    let eight = builder.constant(Fp::from(8));

    let b = builder.add(&a, &one); 

    let c = builder.init();
    let c_times_8 = builder.mul(&c, &eight);

    builder.set(&a, Fp::from(13));
    builder.set(&c, Fp::from(2));

    builder.fill_nodes();
    builder.assert_equal(&c_times_8, &b);

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

    let b = builder.add(&a, &one); 

    let c = builder.hint(&[&b], lambda_div8);
    let c_times_8 = builder.mul(&c, &eight);

    builder.set(&a, Fp::from(13));

    builder.fill_nodes();
    builder.assert_equal(&c_times_8, &b);

    let constraint_check = builder.check_constraints().await; 

    println!("{:?}", constraint_check); 
    println!("{:?}", c_times_8);
    println!("{:?}", b);
}




