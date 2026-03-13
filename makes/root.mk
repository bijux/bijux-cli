ROOT_MK_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))

include $(ROOT_MK_DIR)/_internal.mk
include $(ROOT_MK_DIR)/_macro.mk
include $(ROOT_MK_DIR)/dev-rust.mk
include $(ROOT_MK_DIR)/dev-python.mk
include $(ROOT_MK_DIR)/docs.mk
include $(ROOT_MK_DIR)/gh.mk
