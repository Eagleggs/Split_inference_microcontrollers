import torch
from torchviz import make_dot

# Define a simple computation graph
x = torch.tensor([2.0], requires_grad=True)
y = x ** 2
z = 2 * y

# Define a loss
loss = z.mean()

# Visualize the computational graph
dot = make_dot(loss, params=dict(x=x))
dot.render("computational_graph", format="png")