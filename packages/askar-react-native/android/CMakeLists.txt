cmake_minimum_required(VERSION 3.9.0)
project(ReactNativeAskar)

set(CMAKE_VERBOSE_MAKEFILE ON)
set(CMAKE_CXX_STANDARD 20)

file (GLOB askar-dir "${CMAKE_SOURCE_DIR}/../native/mobile/android/${ANDROID_ABI}")

find_library(
  askar-lib
  # TODO(rename): when lib has changed
  aries_askar 
  paths ${askar-dir}
  NO_CMAKE_FIND_ROOT_PATH
)


find_package(ReactAndroid REQUIRED CONFIG)
add_library(jsi ALIAS ReactAndroid::jsi)
add_library(reactnative ALIAS ReactAndroid::reactnative)

find_package(fbjni REQUIRED CONFIG)
add_library(fbjni ALIAS fbjni::fbjni)

include_directories(
  ../cpp
  ../cpp/include
  ${askar-lib}
)

add_library(react-native-askar
  SHARED
  cpp-adapter.cpp
  ../cpp/HostObject.cpp
  ../cpp/turboModuleUtility.cpp
  ../cpp/askar.cpp
)

target_include_directories(react-native-askar
  PRIVATE
        "${NODE_MODULES_DIR}/react-native/ReactAndroid/src/main/jni/react/turbomodule"
        "${NODE_MODULES_DIR}/react-native/ReactCommon"
        "${NODE_MODULES_DIR}/react-native/ReactCommon/callinvoker"
        "${NODE_MODULES_DIR}/react-native/ReactCommon/jsi"
        "${NODE_MODULES_DIR}/react-native/ReactCommon/runtimeexecutor"
)

target_link_libraries(react-native-askar
  ${askar-lib}
  fbjni
  jsi
  reactnative
  android
)
