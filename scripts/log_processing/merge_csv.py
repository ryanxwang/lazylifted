from argparse import ArgumentParser
import csv

def parse_args():
    parser = ArgumentParser(description="Merge CSV files")
    parser.add_argument("--output", type=str, help="Output CSV file")
    parser.add_argument("input_files", type=str, nargs="+", help="Input CSV files")
    return parser.parse_args()


def main():
    args = parse_args()

    header = None
    new_file = []
    for input_file in args.input_files:
        with open(input_file, "r") as input_csv:
            reader = csv.reader(input_csv)
            if not header:
                new_file.append(next(reader))
                header = True
            else:
                next(reader)
            
            for row in reader:
                new_file.append(row)
    
    with open(args.output, "w") as output_csv:
        writer = csv.writer(output_csv)
        writer.writerows(new_file)


if __name__ == "__main__":
    main()