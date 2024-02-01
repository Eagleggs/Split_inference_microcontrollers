import torch
import torch.nn as nn

# Input and parameters
input_value = torch.tensor([[-0.15519893728196621]])
bias = torch.tensor([-0.084354020655155181])
weight = torch.tensor([0.038120802491903305])
running_variance = torch.tensor([0.9011712074279784])
mean = torch.tensor([-0.48183417320251465])
# Manual batch normalization calculation
variance = torch.max(running_variance, torch.tensor(1e-5))
output_manual = weight * (input_value - mean) / torch.sqrt(variance) + bias
print("Manual Batch Norm Output:", output_manual.item())
