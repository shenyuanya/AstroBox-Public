package moe.astralsight.astrobox
import android.os.Bundle
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback

class MainActivity : TauriActivity(){
    private var webView: WebView? = null;
  
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                webView?.let { wv -> 
                    if (wv.canGoBack()) {
                        wv.goBack()
                    } else {
                        remove()
                        onBackPressedDispatcher.onBackPressed()
                    }
                } ?: run {
                    remove()
                    onBackPressedDispatcher.onBackPressed()
                }
            }
        })
    }

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        this.webView = webView
    }
}