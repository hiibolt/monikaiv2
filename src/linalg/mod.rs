pub fn magnitude( input: &Vec<f64> ) -> f64 {
    let elements_squared = input
        .iter()
        .map(|element| element.powi(2))
        .collect::<Vec<f64>>();
    
    let elements_sum = elements_squared
        .iter()
        .sum::<f64>();
    
    let sqrt_element_sum = elements_sum.sqrt();
    
    sqrt_element_sum
}

pub fn dot_product( input_1: &Vec<f64>, input_2: &Vec<f64> ) -> f64 {
    if input_1.len() != input_2.len() {
        panic!("Incompatible vectors!");
    }

    let like_indices_mulitiplied = input_1
        .iter()
        .enumerate()
        .map(|(element_index, element)| element * input_2[element_index] )
        .collect::<Vec<f64>>();

    let sum_all = like_indices_mulitiplied
        .iter()
        .sum::<f64>();

    sum_all
}

pub fn cosine_similarity( input_1: &Vec<f64>, input_2: &Vec<f64> ) -> f64 {
    dot_product( input_1, input_2 ) / ( magnitude( input_1 ) + magnitude( input_2 ) )
}