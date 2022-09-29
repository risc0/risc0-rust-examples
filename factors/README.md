# Factors

The _factors_ example is a minimalistic RISC Zero zkVM proof. The prover demonstrates that they know two nontrivial factors (i.e. both greater than 1) of a number, without revealing what those factors are. Thus, the prover demonstrates that a number is composite -- and that they know the factors -- without revealing any further information about the number.

This example was chosen because it is very straightforward. Implementing the verified multiplication and reporting the result in the receipt involves no complexity beyond what is necessary to run the zkVM at all. We therefore hope this example is a good place to look to see all the steps necessary to use the RISC Zero zkVM without any superfluous problem-specific details.

Choosing a simple example necessarily excludes all of the more complex use cases -- so if you are looking for anything beyond the basics, we recommend looking at other examples in this repository!

## Run this example

To run, simply use the command
```
cargo run
```
