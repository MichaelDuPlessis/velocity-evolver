import pandas as pd

def csv_to_latex_table(csv_path):
    # Read the CSV file into a DataFrame
    df = pd.read_csv(csv_path)

    # Define a function to format numbers to 4 decimal places
    def format_decimal(x):
        return '{:.4f}'.format(x)

    # Apply the formatting function to the DataFrame
    df_formatted = df.applymap(format_decimal)

    # Add row numbers as a new column
    df_formatted['Row'] = range(1, len(df) + 1)

    latex_table = df_formatted.to_latex(index=False, escape=False, column_format='cccc')

    # Print the LaTeX table
    print(latex_table)

if __name__ == "__main__":
    csv_to_latex_table("./results_copy/canonical30.csv")
    csv_to_latex_table("./results_copy/canonical100.csv")
