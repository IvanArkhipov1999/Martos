idf_component_register(SRCS "main.c" #INCLUDE_DIRS
#"${CMAKE_CURRENT_LIST_DIR}/../rust-ex/src"
REQUIRES rust-ex)

#target_link_options(${COMPONENT_LIB} PUBLIC
#-Wl,--whole-archive ${CARGO_TARGET_DIR}/librust_ex.a -Wl,--no-whole-archive)