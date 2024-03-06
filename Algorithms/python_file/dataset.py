import os

import torchvision
from torchvision import transforms, datasets
from torch.utils.data import Subset


def download_and_save_subset(subset_size=500, save_folder='./imagenet_subset'):
    # Specify the path where you want to store the subset
    os.makedirs(save_folder, exist_ok=True)

    # Download and load the ImageNet dataset
    full_dataset = datasets.ImageNet(root='./data', split='val')

    # Create a subset with the specified number of samples
    indices = list(range(subset_size))
    subset = Subset(full_dataset, indices)

    # Save the images to the specified folder
    for i, (image, label) in enumerate(subset):
        save_path = os.path.join(save_folder, f'image_{i + 1}.jpg')
        torchvision.utils.save_image(image, save_path)

    print(f"{subset_size} images saved to {save_folder}")

    return subset


imagenet_subset = download_and_save_subset()
