import torch
import torch.nn as nn
from torchvision.models import mobilenet_v2
import torch
import torch.nn as nn
import json

class IntermediateOutputsHook:
    def __init__(self):
        self.outputs = []
        self.handles = []
        self.inputs = []
        self.modules = []

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
        self.inputs.append(input)
        self.modules.append(module)

    def remove_hooks(self):
        # Remove all the registered hooks
        for handle in self.handles:
            handle.remove()


def trace_weights(hook):
    conv_mapping = []
    linear_mapping = []
    layer_id = 0
    for layer in zip(hook.inputs, hook.outputs, hook.modules):
        layer_id += 1
        if isinstance(layer[2], torch.nn.Conv2d):
            kernel_size = layer[2].kernel_size
            padding = layer[2].padding
            groups = layer[2].groups
            stride = layer[2].stride
            c, h, w = layer[1][0].shape
            b, c1, h1, w1 = layer[0][0].shape
            weights = layer[2].weight.detach().numpy()
            input_per_group = int(c1 / groups)
            output_per_group = int(c / groups)
            for i in range(c):
                map_weights = weights[i, :, :, :].tolist()
                o_i_mapping = f"i_ch_s = o_ch/{output_per_group} * {input_per_group}\n" \
                              f"s = {stride},k = {kernel_size}" \

                conv_mapping.append((map_weights,o_i_mapping))
                # for j in range(h):
                #     for k in range(w):
                #         output_position = (i, j, k)
                #         # Calculate offsets on the input
                #         h_offset = j * stride[0]
                #         w_offset = k * stride[1]
                #         which_group = int(i / output_per_group) * input_per_group
                #         input_positions = []
                #         map_weights = []
                #         for q in range(input_per_group):
                #             for m in range(kernel_size[0]):
                #                 for n in range(kernel_size[1]):
                #                     # Todo: store mapping of output to input instead of locations
                #                     input_positions.append((
                #                                            which_group * input_per_group + q, h_offset + m - padding[0],
                #                                            w_offset + n - padding[1]))
                #                     map_weights.append(weights[i, q, m, n])
                #         conv_mapping.append((layer_id, input_positions, output_position))

        # if isinstance(layer[2], torch.nn.Linear):
        #     b_in, c_in = layer[0][0].shape
        #     b_out, c_out = layer[1].shape
        #     for i in range(b_out):
        #         for j in range(c_out):
        #             output_position = (i, j)
        #             input_positions = []
        #             map_weights = []
        #             bias = layer[2].bias[j].detach().numpy()
        #             for m in range(c_in):
        #                 input_positions.append((i, m))
        #                 map_weights.append(layer[2].weight[j, m].detach().numpy())
        #             linear_mapping.append((layer_id, input_positions, map_weights, bias, output_position))
        print(f"layer {layer_id} finished")
    return conv_mapping

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
mapping = trace_weights(hook)
with open('serialized_list.json', 'w') as file:
    json.dump(mapping, file)

print("-----")
# Remove the hooks after you're done
hook.remove_hooks()
