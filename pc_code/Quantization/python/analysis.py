import json

import numpy as np
from matplotlib import pyplot as plt
import math


def dot_product(v1, v2):
    return sum(x * y for x, y in zip(v1, v2))


def norm(v):
    return math.sqrt(sum(x * x for x in v))


def cosine_similarity(v1, v2):
    if len(v1) != len(v2):
        raise ValueError("Vectors must have the same length")

    dot_product_value = dot_product(v1, v2)
    norm_v1 = norm(v1)
    norm_v2 = norm(v2)

    return dot_product_value / (norm_v1 * norm_v2)


def euclidean_distance(v1, v2):
    if len(v1) != len(v2):
        raise ValueError("Vectors must have the same length")
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(v1, v2)))


scales = [0.017818455, 0.010181317, 0.023514507, 0.041469865, 0.019598939, 0.016114194, 0.026925236, 0.0057206596,
          0.011403129, 0.040337086, 0.0084821, 0.009201978, 0.021326661, 0.0041799145, 0.006351808, 0.026975844,
          0.004276918, 0.0064519304, 0.03300309, 0.0061818487, 0.007596627, 0.017282467, 0.0030426302, 0.0047031413,
          0.019367196, 0.0027530068, 0.0051408643, 0.020728296, 0.002869781, 0.00812551, 0.022537425, 0.004070429,
          0.007840167, 0.016347256, 0.0041988418, 0.00890296, 0.019542953, 0.0057104784, 0.008726812, 0.026837224,
          0.0054687196, 0.008290729, 0.013501433, 0.005058123, 0.0073697055, 0.019419255, 0.0048613106, 0.0076185213,
          0.0346372, 0.0035971922, 0.004404244, 0.0094860755, 0.023512602, 0.09354488]
zero_points = [114, 0, 0, 130, 0, 0, 117, 0, 0, 121, 0, 0, 136, 0, 0, 132, 0, 0, 135, 0, 0, 132, 0, 0, 130, 0, 0, 131,
               0, 0, 132, 0, 0, 126, 0, 0, 126, 0, 0, 124, 0, 0, 133, 0, 0, 126, 0, 0, 126, 0, 0, 128, 0, 88.16121]

# Open the JSON file

with open(r'.\python\intermediate_o.json', 'r') as file1, open(r".\python\intermediate_q.json") as file2:
    # Read each line
    id = 1
    for line1, line2 in zip(file1, file2):
        # Parse the line as JSON
        s = scales[id - 1]
        z = zero_points[id - 1]
        f32_vector = json.loads(line1)
        u8_vector = json.loads(line2)
        layer16_qo = [(float(x) - z) * s for x in u8_vector]
        if id > 100:
            # Create subplots
            fig, axs = plt.subplots(2, dpi=100)

            # Plot vector1
            axs[0].plot(f32_vector, marker='o', color='blue')
            axs[0].set_title('Vector 1')
            axs[0].set_xlabel('Index')
            axs[0].set_ylabel('Value')

            # Plot vector2
            axs[1].plot(layer16_qo, marker='o', color='green')
            axs[1].set_title('Vector 2')
            axs[1].set_xlabel('Index')
            axs[1].set_ylabel('Value')

            # Adjust layout
            plt.tight_layout()

            # Display the plot
            plt.show()
            plt.clf()
        cosine_sim = cosine_similarity(layer16_qo, f32_vector)
        euclidean_dist = euclidean_distance(layer16_qo, f32_vector)
        vector1 = np.array(layer16_qo)
        vector2 = np.array(f32_vector)

        # Compute Pearson correlation coefficient
        pearson_corr = np.corrcoef(vector1, vector2)[0, 1]

        count_above_threshold = sum(1 for value in f32_vector if value > (255 - z) * s)
        count_below_threshold = sum(1 for value in f32_vector if value < (0 - z) * s)
        t = 0
        for q, o in zip(layer16_qo, f32_vector):
            t += abs(o - q) / s
        t /= len(layer16_qo)
        print(f"id:{id} Cosine Similarity:{cosine_sim},person_corr:{pearson_corr},size: {len(f32_vector)}")
        print(f"t: {t}")
        print(max(f32_vector), max(u8_vector), max(layer16_qo))
        print(min(f32_vector), min(u8_vector), min(layer16_qo))
        print(f"above threshold : {count_above_threshold},below threshold: {count_below_threshold}")
        print("-------------------")
        id += 1
