idf_component_register(SRCS "hello_world_main.c"
                    INCLUDE_DIRS "")

# Linking with martos static library. Set here path to it in your project.
# Here path is so for ci.
# TODO: should be more beautiful linking
target_link_libraries(${COMPONENT_LIB} ${CMAKE_CURRENT_LIST_DIR}/../../../../c-library/xtensa-esp32/target/xtensa-esp32-none-elf/debug/libxtensa_esp32_static_lib.a -Wl,--allow-multiple-definition)
