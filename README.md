# Cirle Fill

1. Generate a random color
2. Find three pixels with very similar colors
3. Find the circle that goes through those three pixels
4. Walk along the circle from the middle pixel towards the closer of the other two pixels
5. If an empty pixel is found, put the color there.
6. If a fraction of the circle is traversed and nothing is found, try a new color

![A programmatically generated artwork, in a rainbow variety of colors, with a pink-yellow art in the top-right, a yellow-red splash in the center top, a white-blue-pink splash in the bottom right, an aqua strak along the bottom, and more diffuse rainbow colors in the middle left.](https://github.com/isaacg1/circle-fill/blob/main/1000-100-20-0.01-10000-0.png)
