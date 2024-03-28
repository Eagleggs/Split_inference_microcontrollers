import matplotlib.pyplot as plt

# Given data
number_of_cpus = [1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120]
max_input_size = [4788.375, 1764.0, 1764.0, 1411.3125, 1058.625, 847.125, 784.0, 784.0, 784.0, 784.0, 752.6875, 684.25, 627.25]
max_weight_size = [1682.5, 169.5644, 85.43945, 57.835938, 43.376953, 35.490234, 30.232422, 26.289063, 22.345703, 21.03125, 19.072266, 17.08789, 15.7734375]
total_weight_size = [9653.0625, 9993.305, 10371.352, 10749.398, 11127.445, 11505.492, 11883.539, 12261.586, 12639.633, 13017.68, 13395.727, 13763.83, 14141.877]
ram_usage = []
# Function to add labels to the points on the lines
def add_labels(x, y, ax):
    for i, txt in enumerate(y):
        ax.text(x[i], txt, f'{txt:.2f}', ha='center', va='bottom')
total_ram_usage = max_weight_size.copy()
for e in range(len(max_weight_size)):
    max_weight_size[e] = max(max_weight_size[e],1280 * 1000 * 4 / 1024 / number_of_cpus[e])
    total_ram_usage[e] = max_weight_size[e] + max_input_size[e]
# Plotting Maximum Input Size in a separate plot
plt.figure(figsize=(10, 6))
ax1 = plt.subplot(111)

plt.plot(number_of_cpus, max_input_size, label='Maximum Input Size', color='blue')
add_labels(number_of_cpus, max_input_size, ax1)

# Adding labels and title
plt.xlabel('Number of CPUs')
plt.ylabel('Size (KB)')
plt.title('Maximum Input Size with Number of CPUs')

# Adding a legend
plt.legend()

# Show the plot
plt.show()

# Plotting Maximum Weight Size in a separate plot
plt.figure(figsize=(10, 6))
ax2 = plt.subplot(111)

plt.plot(number_of_cpus, max_weight_size, label='Maximum Weight Size', color='green')
add_labels(number_of_cpus, max_weight_size, ax2)

# Adding labels and title
plt.xlabel('Number of CPUs')
plt.ylabel('Size (KB)')
plt.title('Maximum Weight Size with Number of CPUs')

# Adding a legend
plt.legend()

# Show the plot
plt.show()

# Plotting Total Weight Size in a separate plot
plt.figure(figsize=(10, 6))
ax3 = plt.subplot(111)

plt.plot(number_of_cpus, total_weight_size, label='Total Weight Size', color='orange')
add_labels(number_of_cpus, total_weight_size, ax3)

# Adding labels and title
plt.xlabel('Number of CPUs')
plt.ylabel('Size (KB)')
plt.title('Total Weight Size with Number of CPUs')

# Adding a legend
plt.legend()

# Show the plot
plt.show()

# Plotting Maximum Weight Size in a separate plot
plt.figure(figsize=(10, 6))
ax2 = plt.subplot(111)

plt.plot(number_of_cpus, total_ram_usage, label='maximum ram usage', color='black')
add_labels(number_of_cpus, total_ram_usage, ax2)

# Adding labels and title
plt.xlabel('Number of CPUs')
plt.ylabel('Size (KB)')
plt.title('maximum ram usage with Number of CPUs')

# Adding a legend
plt.legend()

# Show the plot
plt.show()