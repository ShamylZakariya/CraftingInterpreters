project('clox', 'c', version: '0.1', default_options : [
    'c_std=c11'])


# defines clox_lib
subdir('libclox')

# build the main executable
app_sources = [
    'main.c'
]

executable('clox',
    sources: app_sources,
    link_with: clox_lib)
