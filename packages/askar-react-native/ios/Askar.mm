#import <React/RCTBridge+Private.h>
#import <jsi/jsi.h>
#import <React/RCTUtils.h>
#import <ReactCommon/RCTTurboModule.h>
#import <askar/turboModuleUtility.h>

#import "Askar.h"

using namespace facebook;

@implementation Askar 

RCT_EXPORT_MODULE()

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(install)
{
    RCTBridge* bridge = [RCTBridge currentBridge];
    RCTCxxBridge* cxxBridge = (RCTCxxBridge*)bridge;
    if (cxxBridge == nil) {
        return @false;
    }
    
    jsi::Runtime* jsiRuntime = (jsi::Runtime*) cxxBridge.runtime;
    if (jsiRuntime == nil) {
        return @false;
    }
    
    auto callInvoker = bridge.jsCallInvoker;
    askarTurboModuleUtility::registerTurboModule(*jsiRuntime, callInvoker);
    return @true;
}

@end
