# saavn-cli
Jio saavn cli for rust which can search and download songs in 320kbps

```
Usage: saavn-cli [OPTIONS] <ACTION>

Arguments:
  <ACTION>  [possible values: search, download, play]

Options:
  -n, --name <NAME>  
  -h, --help         Print help information
  -V, --version      Print version information
```
## Example

```
saavn-cli search --name "<name>"
```
to get top results, select the name with return/enter key and then select play or download action
