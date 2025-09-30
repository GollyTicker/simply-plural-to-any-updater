# TODO

We want to add custom prometheus metrics for a few important aspects. Checkout simply_plural.rs and metrics.rs on how they're defined, used and registered.

Then search the `src` folder for lines of the form
```
// counter MY_METRIC_NAME label1, label2, ...
```

or `// gauge MY_METRIC_NAME`

and 

## STEPS

