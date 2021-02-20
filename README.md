# Lupinas Lullaby

Parse Swagger files (v2 and v3) into a Typescript interface file.

## Help

```
lupinas-lullaby 0.1.4
Hanif Bin Ariffin <hanif.ariffin.4326@gmail.com>
Automatically generate TypeScript interfaces from a Swagger spec file.

USAGE:
    lupinas-lullaby [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --auth_password <auth_password>    The basic authentication username payload to pass along.
        --auth_user <auth_user>            The basic authentication password payload to pass along.
        --file <file>                      The Swagger file to parse.
        --outfile <outfile>                The destination file to write to.
                                           If this value is not specified, it will simply write to stdout.
        --url <url>                        The URL to the Swagger file. Must be a URL to a JSON/YAML resource
```

## Example

It doesn't have the best format ... but its a perfectly valid TS file. You will `prettier` this away anyways.

```
lupinas-lullaby on  master is 📦 v0.1.3 via ⬢ v15.2.0 via 🦀 v1.49.0
❯ lupinas-lullaby --url https://petstore.swagger.io/v2/swagger.json
export type ApiResponse = {"code" ? : number;"message" ? : string;"type" ? : string;}
export type Category = {"id" ? : number;"name" ? : string;}
export type Order = {"complete" ? : boolean;"id" ? : number;"petId" ? : number;"quantity" ? : number;"shipDate" ? : Date;"status" ? : "placed" |"approved" |"delivered" ;}
export type Pet = {"category" ? : Category;"id" ? : number;"name"  : string;"photoUrls"  : string[];"status" ? : "available" |"pending" |"sold" ;"tags" ? : Tag[];}
export type Tag = {"id" ? : number;"name" ? : string;}
export type User = {"email" ? : string;"firstName" ? : string;"id" ? : number;"lastName" ? : string;"password" ? : string;"phone" ? : string;"userStatus" ? : number;"username" ? : string;}
```
