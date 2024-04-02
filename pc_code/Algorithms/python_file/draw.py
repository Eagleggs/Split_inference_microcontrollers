import matplotlib.pyplot as plt

# Given data
number_of_cpus = [1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120]
max_input_size = [4788.375, 1764.0, 1764.0, 1411.3125, 1058.625, 847.125, 784.0, 784.0, 784.0, 784.0, 752.6875, 684.25, 627.25]
max_weight_size = [1682.5, 169.5644, 85.43945, 57.835938, 43.376953, 35.490234, 30.232422, 26.289063, 22.345703, 21.03125, 19.072266, 17.08789, 15.7734375]
total_weight_size = [9653.0625, 9993.305, 10371.352, 10749.398, 11127.445, 11505.492, 11883.539, 12261.586, 12639.633, 13017.68, 13395.727, 13763.83, 14141.877]

ram_usage = [5973.9375,1795.9102,1780.584,1422.3672,1067.2305,878.4629,862.7832,851.584,843.18164,836.6504,799.9844,727.26953,666.70703]
# number_of_mcus = [1,2,3,4,5,6,10,20,40]
# time_spent_on_calculation = [2.4623916,1.3346502,0.8617504,0.6617702,0.5666856,0.5030999,0.4276457,0.2534328,0.1953952]
# total_time = [9.5053,6.808295,6.1875686,6.3416881,8.9093638,8.9801163,20.686958,69.0034362,110.3210564]
number_of_mcus = [1,2,3,4,5,6,10]
time_spent_on_calculation = [2.4623916,1.3346502,0.8617504,0.6617702,0.5666856,0.5030999,0.4276457]
total_time = [9.5053,6.808295,6.1875686,6.3416881,8.9093638,8.9801163,20.686958]
time_spent_on_communication = [total_time[i] - time_spent_on_calculation[i] for i in range(len(number_of_mcus))]
for i in range(0,len(number_of_mcus)):
    time_spent_on_communication[i] = total_time[i] - time_spent_on_calculation[i]
# Function to add labels to the points on the lines
# Plotting
plt.figure(figsize=(10, 6))

# Plot time spent on calculation
plt.plot(number_of_mcus, time_spent_on_calculation, label='Time Spent on Calculation', marker='o')

# Plot time spent on communication
plt.plot(number_of_mcus, time_spent_on_communication, label='Time Spent on Communication', marker='o')

# Plot total time
plt.plot(number_of_mcus, total_time, label='Total Time', marker='o')
for i, mcu in enumerate(number_of_mcus):
    plt.annotate(f'{time_spent_on_calculation[i]:.2f}', (mcu, time_spent_on_calculation[i]), textcoords="offset points", xytext=(0,10), ha='center')
    plt.annotate(f'{time_spent_on_communication[i]:.2f}', (mcu, time_spent_on_communication[i]), textcoords="offset points", xytext=(0,10), ha='center')
    plt.annotate(f'{total_time[i]:.2f}', (mcu, total_time[i]), textcoords="offset points", xytext=(0,10), ha='center')

# Adding labels and legend
plt.xlabel('Number of MCUs')
plt.ylabel('Time(s)')
plt.title('Time Metrics')
plt.legend()

# Show plot
plt.grid(True)
plt.show()
def add_labels(x, y, ax):
    for i, txt in enumerate(y):
        ax.text(x[i], txt, f'{txt:.2f}', ha='center', va='bottom')
for e in range(len(max_weight_size)):
    max_weight_size[e] = max(max_weight_size[e],1280 * 1000 * 4 / 1024 / number_of_cpus[e])
    ram_usage[e] = max(1280 * 1000 / 1024 / number_of_cpus[e] + 1280 / 1024 , ram_usage[e] / 4)
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

plt.plot(number_of_cpus, ram_usage, label='maximum ram usage after quantization', color='black')
add_labels(number_of_cpus, ram_usage, ax2)

# Adding labels and title
plt.xlabel('Number of MCUs')
plt.ylabel('Size (KB)')
plt.title('peak ram usage with Number of MCUs')

# Adding a legend
plt.legend()

# Show the plot
plt.show()