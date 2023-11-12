import torch
import torch.nn as nn
from torchvision.models import mobilenet_v2


class IntermediateOutputsHook:
    def __init__(self):
        self.outputs = []
        self.handles = []

    def register(self, model):
        # Register a forward hook for each submodule
        for submodule in model.children():
            if len(list(submodule.children())) == 0:
                handle = submodule.register_forward_hook(self.hook_fn)
                self.handles.append(handle)
            else:
                self.register(submodule)

    def hook_fn(self, module, input, output):
        # Save the intermediate output
        self.outputs.append(output)
        print(input)
    def remove_hooks(self):
        # Remove all the registered hooks
        for handle in self.handles:
            handle.remove()


# Load the pretrained MobileNetV2 model
model = mobilenet_v2(pretrained=True)

# Instantiate the hook
hook = IntermediateOutputsHook()
hook.register(model)
# Dummy input tensor (replace this with your actual input data)
input_data = torch.randn(1, 3, 224, 224)

# Forward pass with the hooked model
output = model(input_data)

# Access the intermediate outputs
intermediate_outputs = hook.outputs

# Print or analyze the intermediate outputs as needed
for i, output in enumerate(intermediate_outputs):
    print(f"Intermediate output {i + 1}: {output.shape}")

# Remove the hooks after you're done
hook.remove_hooks()
