use takehome::{builder::Builder, field::GaloisField};

pub type Fp = GaloisField::<65537>;

// test that f(x) = x^2 + x + 5 is constructed and evaluated correctly 
#[test]
fn test_basic_function() {
    let mut builder = Builder::<Fp>::new();

    let x = builder.init();
    let x_squared = builder.mul(&x, &x);
    let five = builder.constant(Fp::from(5));
    let x_squared_plus_5 = builder.add(&x_squared, &five);
    let y = builder.add(&x_squared_plus_5, &x);

    builder.fill_nodes(vec![Fp::from(5)]);

    assert_eq!(x.read().unwrap().value.unwrap().value, 5);
    assert_eq!(x_squared.read().unwrap().value.unwrap().value, 25);
    assert_eq!(five.read().unwrap().value.unwrap().value, 5);
    assert_eq!(x_squared_plus_5.read().unwrap().value.unwrap().value, 30);
    assert_eq!(y.read().unwrap().value.unwrap().value, 35);
}

// test that multiple threads can read the same value at the same time. 
#[test]
fn test_multiple_access() {
    let mut builder = Builder::<Fp>::new();

    let x = builder.init();
    let y = builder.init();
    let z = builder.init();
    let w = builder.init(); 

    let x2 = builder.mul(&x, &x);
    let xy = builder.mul(&x, &y);
    let xz = builder.mul(&x, &z);
    let xw = builder.mul(&x, &w);

    builder.fill_nodes(vec![Fp::from(5), Fp::from(5), Fp::from(45), Fp::from(6)]);
    assert_eq!(x.read().unwrap().value.unwrap().value, 5);
    assert_eq!(y.read().unwrap().value.unwrap().value, 5);
    assert_eq!(z.read().unwrap().value.unwrap().value, 45);
    assert_eq!(w.read().unwrap().value.unwrap().value, 6);

    assert_eq!(x2.read().unwrap().value.unwrap().value, 25);
    assert_eq!(xy.read().unwrap().value.unwrap().value, 25);
    assert_eq!(xz.read().unwrap().value.unwrap().value, 225);
    assert_eq!(xw.read().unwrap().value.unwrap().value, 30);
}

// test the check_constraints() function
#[tokio::test]
async fn test_constraints() {
    let mut builder = Builder::<Fp>::new();
    let a = builder.init();
    let one = builder.constant(Fp::from(1)); 
    let eight = builder.constant(Fp::from(8));

    let b = builder.add(&a, &one); 

    let c = builder.init();
    let c_times_8 = builder.mul(&c, &eight);

    builder.fill_nodes(vec![Fp::from(13), Fp::from(2)]);
    builder.assert_equal(&c_times_8, &b);

    let constraint_check = builder.check_constraints().await; 

    println!("{:?}", constraint_check); 
    println!("{:?}", c_times_8);
    println!("{:?}", b);
}

// equivalent to the lambda x -> x/8 but written as a "vector-function". 
fn lambda_div8(val: Vec<Fp>) -> Fp {
    assert_eq!(val.len(), 1);
    val[0] / Fp::from(8)
}

// test that the example on the website works 
#[tokio::test]
async fn test_hints() {
    let mut builder = Builder::<Fp>::new();
    let a = builder.init();
    let one = builder.constant(Fp::from(1)); 
    let eight = builder.constant(Fp::from(8));

    let b = builder.add(&a, &one); 

    let c = builder.hint(&[&b], lambda_div8);
    let c_times_8 = builder.mul(&c, &eight);

    builder.fill_nodes(vec![Fp::from(13)]);
    builder.assert_equal(&c_times_8, &b);

    let constraint_check = builder.check_constraints().await; 

    println!("{:?}", constraint_check); 
    println!("{:?}", c_times_8);
    println!("{:?}", b);
}

// the function (a,b) -> a/b
fn lambda_div(params: Vec<Fp>) -> Fp {
    params[0] / params[1]
}

#[tokio::test]
async fn test_lambda_gates() {
    let mut builder = Builder::<Fp>::new();

    let a = builder.init();
    let b = builder.init();

    let c = builder.mul(&a, &b);

    let d = builder.hint(&[&c, &b], lambda_div);

    builder.assert_equal(&d, &a);
    builder.fill_nodes(vec![Fp::from(234), Fp::from(123)]);
    let passed_constraints = builder.check_constraints().await; 

    assert!(passed_constraints);
    assert_eq!(a.read().unwrap().value.unwrap().value, 234);
    assert_eq!(b.read().unwrap().value.unwrap().value, 123);
    assert_eq!(c.read().unwrap().value.unwrap().value, 28782);
    assert_eq!(d.read().unwrap().value.unwrap().value, 234);
}
