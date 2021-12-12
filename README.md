# Conjure
### About

Conjure is a 3D Constructive Solid Geometry (CSG) language.

Rough prototype at the moment.

### Example Conjure Lang

```
(union
  ; centered on origin
  (sphere 4)
  ; lower left point & upper right point
  (cube [-3 -3 -3] [3 3 3])
)
```
![rendering of union](examples/union.png)
