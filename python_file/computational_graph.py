import torch
from torchviz import make_dot
from torchvision.models import mobilenet_v2

# Load the pretrained MobileNetV2 model
model = mobilenet_v2(pretrained=True)

# Dummy input tensor (replace this with your actual input data)
input_data = torch.randn(1, 3, 224, 224)

# Run a forward pass
output = model(input_data)

# Visualize the computational graph
dot = make_dot(output, params=dict(model.named_parameters()))
dot.render("mobilenetv2_computational_graph", format="png")
