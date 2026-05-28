# ELP2000 JSON schema

See `schema/elp2000.schema.json` for the JSON Schema.

## Series naming

ELP2000-82B defines 36 series (ELP1–ELP36), grouped into three types:

| Type | Series | Structure |
|------|--------|-----------|
| Main problem (`MainProblem`) | ELP1–ELP3 | `ilu[4]`, `a`, `b[6]` |
| Earth–Moon figure perturbations (`EarthPert`) | ELP4–ELP9, ELP22–ELP30, ELP33–ELP36 | `iz`, `ilu[4]`, `o`, `a`, `p` |
| Planetary perturbations (`PlanetPert`) | ELP10–ELP21, ELP31–ELP32 | `ipla[11]`, `theta`, `o`, `p` |

## JSON layout

```json
{
  "source": "ELP2000-82B (Chapront-Touzé & Chapront 1988)",
  "series": {
    "ELP1": [
      {"type": "main", "ilu": [0,0,0,2], "a": -411.60287, "b": [168.48, 0.0, 0.0, 0.0, 0.0, 0.0]},
      ...
    ],
    "ELP10": [
      {"type": "planet", "ipla": [0,0,0,0,0,1,0,0,0,0,0], "theta": 0.0, "o": 0.0, "p": 0.0},
      ...
    ]
  }
}
```
