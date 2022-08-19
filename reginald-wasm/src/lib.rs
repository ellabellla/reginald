use wasm_bindgen::prelude::*;
use reginald_lib::*;

#[wasm_bindgen]
pub struct Regex {
    internal: regex::Regex
}

#[wasm_bindgen]
pub struct Slice {
    pub start: usize,
    pub size: usize,
}

impl Slice {
    pub fn new(slice: (usize, usize)) -> Slice {
        Slice{start: slice.0, size: slice.1}
    }
}


#[wasm_bindgen]
pub struct Slices {
    slices: Vec<(usize, usize)>,
}

#[wasm_bindgen]
impl Slices {
    pub fn get(&self, index: usize) -> Option<Slice> {
        self.slices.get(index).map(|slice| Slice::new(*slice))
    }

    pub fn len(&self) -> usize {
        self.slices.len()
    }
}

fn new_slices(slices: Vec<(usize, usize)>) -> Slices {
    Slices{slices}
}


#[wasm_bindgen]
impl Regex {
    #[wasm_bindgen(constructor)]
    pub fn compile(code: &str) -> Result<Regex, String> {
        match regex::Regex::compile(code) {
            Ok(regex) => Ok(Regex{internal: regex}),
            Err(err) => Err(err.to_string()),
        }
    }
    pub fn test(&self, string: &str) -> bool {
        self.internal.test(string)
    }

    pub fn matches(&self, string: &str) -> Slices {
        new_slices(self.internal.matches(string))
    }

    pub fn is_match(&self, string: &str) -> Option<Slice> {
        self.internal.is_match(string).map(|slice| Slice::new(slice))
    }

    pub fn to_string(&self) -> String {
        self.internal.to_string()
    }
}