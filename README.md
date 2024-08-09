 # File structure
 This repository contains the code for both the __PC side__ and the __MCU side__ for my master thesis. 
 ## The PC code
 ### Algorithms
 __images__: This folder contains some test images for the project.  
 
 __json_files__: This folder contains the recorded model files.  
 
 __python_file__: This folder contains the algorithms used to convert the original pytorch CNN models (in my case MobilenetV2 specifically) to json files, you can simply run `hook.py` to insert hooks in the model and then record the models in a json file.
 You can specify the number of layers you want to record in the line of `if layer_id == ?`. `draw.py` contains the code I used to produce some of my data image for my thesis paper.  

 __src__: This folder contains the Rust code I used to implement the project. In `lib.rs`, I implemented a simple yet effictive Rust representation of CNN models focusing on some basic functionalities
 like forward propagation and receptive field determination while making use of Rust's traits. In `calculation.rs`, I implemented some calculation
functions which are only used for testing. In `decode.rs`,
I implemented the function used to interpret the converted
 json files into Rust objects which are defined in `lib.rs`. In 
`operations.rs`, I implemented the algorithms used to distribute the models,
as well as the method I used to do convolution using a flattened 1-D vector and weights data.
 I
