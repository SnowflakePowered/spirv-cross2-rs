import sys;
import re;

def main():
    try:
        filename = sys.argv[1]
    except:
        print("Missing file argument.", file=sys.stderr)
        return 1

    with open(filename) as file:
        while line := file.readline():
            full = line.strip()
            matches = re.split("^(SPVC_COMPILER_OPTION_)([A-Z]+_)([0-9A-Z_]+)$", full)
            name = matches[3].lower()
            txt = f"""
    ///
    #[option({full}, false)]
    pub {name}: bool,"""

            print(txt)

    
if __name__ == "__main__":
    main()