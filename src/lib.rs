use filter::FilterBuilder;

// ! Sieve is a real-time data streaming and filtering engine for ethereum & the superchain
mod filter;

fn main() {
    let filter = FilterBuilder::new()
    .tx(|f| {
        // value must be greater than 100
        f.value().gt(100);

        // gas price must be greater than 100
        f.gas_price().gt(100);
    })
    .build();

    let f = FilterBuilder::new().any_of(|f| {
        f.any_of(|ff| {
            ff.any_of(|fff| {
                fff.tx(|ffff| {
                    
                    // value must be greater than 100
                    ffff.value().gt(100);
            
                    // gas price must be greater than 100
                    ffff.gas_price().gt(100);
                });

                fff.event(|ffff| {
                    
                });
            });
        });
    })
    .and(|f| {

    })
    .build();

    let f = FilterBuilder::new()
    .any_of(|f| {
        f.tx(|ff| {
            // value must be greater than 100
            ff.value().gt(100);
        });

        f.tx(|ff| {
            // gas price must be greater than 100
            ff.gas_price().gt(100);
        });
    })
    .build();
}