* Location, Contents
* Error reporting: when ;-invoked a Rhai call fails with a `crate::error::Error`, we currently only print the error representation. This is not TOO bad, but we'll definitely need to include a stack trace once verbs are in place. It may then be possible to print the same way in the ; scenario.