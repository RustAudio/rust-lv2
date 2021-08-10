# Metro

This plugin showcases several interesting topics: First of all, it shows how to synthesize notes using a sampled sine wave and an envelope. These notes are also synchronized to the host's transport via the "time" extension and lastly, it uses the [pipes library](https://github.com/Janonard/pipes) to express the internal processing pipeline.

A pipe is similar to an iterator as it has a `next` method that produces the next item of the pipeline. However, it also takes an input item to create this output item. Therefore, individual pipes can be chained into larger pipes and even complete pipelines.

Using pipes has multiple advantages over writing the processing algorithm "manually": First of all, it slices the pipeline into well-defined pieces that can easily be understood on their own. Then, they also provide a testable interface to the individual parts of the algorithm, which is very useful since you can't properly test your code online, and lastly, they also improve the reusability of your code.

However, they also have some downsides: First of all, they require more code than a "manual" implementation, since every pipe is a type on its own. Also, since the algorithm is split into many small methods, there is an overhead from the function calls and it might be hard for the compiler to use [SIMD instructions](https://en.wikipedia.org/wiki/SIMD).

We don't tell you which approach to use, but we would like to show you both so you can decide!