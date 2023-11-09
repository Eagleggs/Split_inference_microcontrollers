import torch
import torch.nn as nn

class SimpleModel(nn.Module):
    def __init__(self):
        super(SimpleModel, self).__init__()
        self.fc1 = nn.Linear(10, 5)
        self.relu = nn.ReLU()
        self.fc2 = nn.Linear(5, 1)

    def forward(self, x):
        x = self.fc1(x)
        x = self.relu(x)
        x = self.fc2(x)
        return torch.sigmoid(x)  # Apply a sigmoid activation to produce a scalar output

# Create an instance of the model
model = SimpleModel()

# Define some input data
input_data = torch.randn(3, 10, requires_grad=True)  # Set requires_grad to True to track gradients

# Perform the forward pass
output = torch.sum(model(input_data))

# Access the computation graph and build a dependency tree
# (backward() is called implicitly during backward pass)
output.backward()

# Print the computation graph and dependencies
print("Computation Graph:\n", torch.autograd.graph(output.grad_fn))
print("\nDependencies:")
for param in model.parameters():
    print(param, param.grad)
