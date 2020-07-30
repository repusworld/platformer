# A project to experiment with ggez
this should turn into a game some day

# Things I learned

Below are things I learned while working on this.

# Calculating the coordinates at which to draw an object, according to a camer and zoom

```cpp
struct vector2
{
    float x, y;
};

/// This function calculates the coordinates to render an object relative to the camera with zoom
/// @param camera_center the absolute coordinates for the center of the camera
/// @param object the absolute coordinates for the object
/// @returns the coordinates at which the object should be drawn relative to the camera with the given zoom
vector2 get_relative_coordinates(vector2 &camera_center, vector2 &object)
{
    vector2 relative;

    relative.x = (object.x - camera_center.x) * zoom + (SCREE_WIDTH / 2);
    relative.y = (object.y - camera_center.y) * zoom + (SCREE_HEIGHT / 2);
    return relative;
}
```

## How i got here

- ### old algorithm, found by trial and error
```txt
// s = screen width
// x = object x
// c = center camera
// z = zoom
```
```matlab
(s/2)+((x-(c-(s/2)))*z)-((s/2)*z)
```

- ### simplified with https://www.dcode.fr/math-simplification
```matlab
z(x−c)+s/2
```

## Key realization

only apply the zoom to the calculated offset from the object to the camera.
after that, move by half the screen to get it centered.