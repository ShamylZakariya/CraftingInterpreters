# Thoughts

Environment::get and LoxInstance::Get return clones of LoxObject; this should be fine since Callable, Class and Instance are all ref-counted types, and it's fine to copy Boolean, Nil, Number, Str and Undefined...

# Challenges

