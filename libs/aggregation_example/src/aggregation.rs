//! Example of an aggregation struct to be used in storage
#[derive(Clone, Debug)]
pub struct AggregationStruct {
    pub count: u32,
    pub total: u32,
}

impl AggregationStruct {
    #[no_mangle]
    pub fn input(&mut self, input_string: String) {
        let val = input_string.parse::<u32>().unwrap();
        self.total += val;
        self.count += 1;
    }

    #[no_mangle]
    pub fn return_value(&mut self) -> String {
        if self.count == 0 {
            return "0".to_string();
        }
        let avg = self.total / self.count;
        return avg.to_string();
    }

    #[no_mangle]
    pub fn init() -> *mut AggregationStruct {
        Box::into_raw(Box::new(AggregationStruct { count: 0, total: 0 }))
    }
}
