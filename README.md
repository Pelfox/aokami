# aokami

CLI for interacting with Minecraft Data Generator. With some transformers.

### Installation

```bash
git clone https://github.com/Pelfox/aokami.git
cd aokami
cargo build --release
# The binary will be in target/release/aokami
```

### Usage

See `aokami --help` for more information.

### Included transformers

aokami comes with some transformers for ease of use.

#### Registries

Converts specified registries into the following format:
```json
{
  "minecraft:registry": {
    "minecraft:subregistry": {},
    "minecraft:another_subregistry": {}
  }
}
```

Run via: `aokami transform registry --registries here,comes,registries`.

> [!NOTE]
> If you need to include a registry that consists of subfolder, specify it like `registry/subregistry`.
> For example: `worldgen/biome`.

##### Common registries (required by protocol)

* `cat_variant`
* `chicken_variant`
* `cow_variant`
* `frog_variant`
* `painting_variant`
* `pig_variant`
* `wolf_sound_variant`
* `wolf_variant`
* `dimension_type`
* `damage_type`
* `worldgen/biome`

#### Blocks

Converts all game blocks into the following format:
```json
{
  "minecraft:air": 0,
  "minecraft:stone": 1
}
```

Run via: `aokami transform blocks`.
