
Use bindgen to generate the rust headers for libvips:

    bindgen wrapper.h 
          --whitelist-function 'vips_.*'
          --whitelist-var 'VIPS_.*'
          --whitelist-type 'Vips.*'
          --whitelist-function 'g_object_.*' > src/lib.rs

