import re

# Open the file
with open('lines.txt', 'r') as file:
    # Read lines from the file
    lines = file.readlines()

# Initialize an empty list to store numbers
numbers = []

# Regular expression pattern to match numbers
pattern = r'\d+'

# Iterate through each line
for line in lines:
    # Find all numbers in the line using regular expression
    nums_in_line = re.findall(pattern, line)
    # Convert the extracted numbers from string to integers
    nums_in_line = [int(num) for num in nums_in_line]
    # Extend the numbers list with the numbers extracted from the current line
    numbers.extend(nums_in_line)

# Print the list of numbers
print(numbers)