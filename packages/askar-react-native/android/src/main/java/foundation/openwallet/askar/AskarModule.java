package foundation.openwallet.askar;

import android.telecom.Call;
import android.util.Log;
import androidx.annotation.Keep;
import androidx.annotation.NonNull;
import androidx.annotation.OptIn;

import com.facebook.proguard.annotations.DoNotStrip;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.common.annotations.FrameworkAPI;
import com.facebook.react.turbomodule.core.CallInvokerHolderImpl;
import com.facebook.react.turbomodule.core.interfaces.CallInvokerHolder;

import java.util.Objects;

import id.animo.SecureEnvironment;

@Keep
@DoNotStrip
public class AskarModule extends ReactContextBaseJavaModule {
    static {
        System.loadLibrary("react-native-askar");
    }

    public static final String NAME = "Askar";

    static String TAG = "Askar";

    public AskarModule(ReactApplicationContext reactContext) {
        super(reactContext);
    }

    @Override
    @NonNull
    public String getName() {
        return TAG;
    }

    private static native void installNative(long jsiRuntimePointer);

    @ReactMethod(isBlockingSynchronousMethod = true)
    public boolean install() {
        try {
            SecureEnvironment.set_env();
            ReactContext context = getReactApplicationContext();
            long jsContextPointer = Objects.requireNonNull(context.getJavaScriptContextHolder()).get();
//            CallInvokerHolderImpl holder = (CallInvokerHolderImpl) context.getCatalystInstance().getJSCallInvokerHolder();
            installNative(jsContextPointer);
            return true;
        } catch (Exception exception) {
            Log.e(NAME, "Failed to install JSI Bindings!", exception);
            return false;
        }
    }
}
