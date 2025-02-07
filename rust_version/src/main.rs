mod t212;

fn main() {
    let mut data = match t212::get_orders() {
        Ok(v) => {println!("Import from t212 successful");
        v
    },
        Err(e) => panic!("Import from t212 failed with error code: {}", e)
    };

    // println!("{:?}", data[0].filledQuantity)
}
