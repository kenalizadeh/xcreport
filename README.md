**A Tool for delivering squad-specific code coverage report.**
> Check releases for the executable binary

```shell
Usage: xctest_rs <COMMAND>

Commands:
  run       Run tests and generate coverage report
  generate  Generate coverage report from test result
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

# Run

## Runs the tests on specified inputs, then generates the report.

INPUT_FILE - Provide an input CSV file which contains two required columns `Squad` and `Filepath`.
PROJECT_PATH - 

```shell
Run tests and generate coverage report

Usage: xctest run \
  --input-file <INPUT_FILE> \
  --project-path <PROJECT_PATH> \
  --workspace <WORKSPACE> \
  --scheme <SCHEME> \
  --destination <DESTINATION>

Options:
  -i, --input-file <INPUT_FILE>      
  -p, --project-path <PROJECT_PATH>  
  -w, --workspace <WORKSPACE>        
  -s, --scheme <SCHEME>              
  -d, --destination <DESTINATION>    
  -h, --help                         Print help
```

# GENERATE

## Generates the report on test result `.xcresult` file.

```shell
Generate coverage report from test result

Usage: xctest generate \
  --input-file <INPUT_FILE> \
  --xcresult-file <XCRESULT_FILE>

Options:
  -i, --input-file <INPUT_FILE>        Path to directory
  -x, --xcresult-file <XCRESULT_FILE>  
  -h, --help                           Print help
```

# OUTPUT

Report consists of a brief `report.csv` and full `full_repot.csv` files. The full report can also be used as `INPUT_FILE` for next iterations.

</br>
</br>


- report.csv
<img width="521" alt="report" src="https://github.com/kenalizadeh/xctest_rs/assets/4370392/fdb023ca-3ecb-4c47-9938-f17bb21eb8c4">

- full_report.csv
<img width="1494" alt="full_report" src="https://github.com/kenalizadeh/xctest_rs/assets/4370392/9181d0df-d0c1-4d74-b14b-fe722543dde9">

