#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveMediaSource(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(AdaptiveMediaSource, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(AdaptiveMediaSource, IClosable, IMediaSource);
impl AdaptiveMediaSource {
    pub fn IsContentTypeSupported(contenttype: &str) -> windows_core::Result<bool> {
        Self::IAdaptiveMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IsContentTypeSupported)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(&windows_core::HSTRING::from(contenttype)),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub fn CreateFromUriAsync<P0>(
        uri: P0,
    ) -> windows_core::Result<windows_future::IAsyncOperation<AdaptiveMediaSourceCreationResult>>
    where
        P0: windows_core::Param<Uri>,
    {
        Self::IAdaptiveMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromUriAsync)(
                windows_core::Interface::as_raw(this),
                uri.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromUriWithDownloaderAsync<P0, P1>(
        uri: P0,
        httpclient: P1,
    ) -> windows_core::Result<windows_future::IAsyncOperation<AdaptiveMediaSourceCreationResult>>
    where
        P0: windows_core::Param<Uri>,
        P1: windows_core::Param<HttpClient>,
    {
        Self::IAdaptiveMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromUriWithDownloaderAsync)(
                windows_core::Interface::as_raw(this),
                uri.param().abi(),
                httpclient.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromStreamAsync<P0, P1>(
        stream: P0,
        uri: P1,
        contenttype: &str,
    ) -> windows_core::Result<windows_future::IAsyncOperation<AdaptiveMediaSourceCreationResult>>
    where
        P0: windows_core::Param<IInputStream>,
        P1: windows_core::Param<Uri>,
    {
        Self::IAdaptiveMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromStreamAsync)(
                windows_core::Interface::as_raw(this),
                stream.param().abi(),
                uri.param().abi(),
                core::mem::transmute_copy(&windows_core::HSTRING::from(contenttype)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromStreamWithDownloaderAsync<P0, P1, P3>(
        stream: P0,
        uri: P1,
        contenttype: &str,
        httpclient: P3,
    ) -> windows_core::Result<windows_future::IAsyncOperation<AdaptiveMediaSourceCreationResult>>
    where
        P0: windows_core::Param<IInputStream>,
        P1: windows_core::Param<Uri>,
        P3: windows_core::Param<HttpClient>,
    {
        Self::IAdaptiveMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromStreamWithDownloaderAsync)(
                windows_core::Interface::as_raw(this),
                stream.param().abi(),
                uri.param().abi(),
                core::mem::transmute_copy(&windows_core::HSTRING::from(contenttype)),
                httpclient.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IAdaptiveMediaSourceStatics<R, F: FnOnce(&IAdaptiveMediaSourceStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<AdaptiveMediaSource, IAdaptiveMediaSourceStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for AdaptiveMediaSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAdaptiveMediaSource>();
}
unsafe impl windows_core::Interface for AdaptiveMediaSource {
    type Vtable = <IAdaptiveMediaSource as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAdaptiveMediaSource as windows_core::Interface>::IID;
}
impl core::ops::Deref for AdaptiveMediaSource {
    type Target = IAdaptiveMediaSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AdaptiveMediaSource {
    const NAME: &'static str = "Windows.Media.Streaming.Adaptive.AdaptiveMediaSource";
}
unsafe impl Send for AdaptiveMediaSource {}
unsafe impl Sync for AdaptiveMediaSource {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveMediaSourceCorrelatedTimes(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AdaptiveMediaSourceCorrelatedTimes,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for AdaptiveMediaSourceCorrelatedTimes {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAdaptiveMediaSourceCorrelatedTimes>();
}
unsafe impl windows_core::Interface for AdaptiveMediaSourceCorrelatedTimes {
    type Vtable = <IAdaptiveMediaSourceCorrelatedTimes as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAdaptiveMediaSourceCorrelatedTimes as windows_core::Interface>::IID;
}
impl core::ops::Deref for AdaptiveMediaSourceCorrelatedTimes {
    type Target = IAdaptiveMediaSourceCorrelatedTimes;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AdaptiveMediaSourceCorrelatedTimes {
    const NAME: &'static str = "Windows.Media.Streaming.Adaptive.AdaptiveMediaSourceCorrelatedTimes";
}
unsafe impl Send for AdaptiveMediaSourceCorrelatedTimes {}
unsafe impl Sync for AdaptiveMediaSourceCorrelatedTimes {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveMediaSourceCreationResult(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AdaptiveMediaSourceCreationResult,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for AdaptiveMediaSourceCreationResult {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAdaptiveMediaSourceCreationResult>();
}
unsafe impl windows_core::Interface for AdaptiveMediaSourceCreationResult {
    type Vtable = <IAdaptiveMediaSourceCreationResult as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAdaptiveMediaSourceCreationResult as windows_core::Interface>::IID;
}
impl core::ops::Deref for AdaptiveMediaSourceCreationResult {
    type Target = IAdaptiveMediaSourceCreationResult;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AdaptiveMediaSourceCreationResult {
    const NAME: &'static str = "Windows.Media.Streaming.Adaptive.AdaptiveMediaSourceCreationResult";
}
unsafe impl Send for AdaptiveMediaSourceCreationResult {}
unsafe impl Sync for AdaptiveMediaSourceCreationResult {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AdaptiveMediaSourceCreationStatus(pub i32);
impl AdaptiveMediaSourceCreationStatus {
    pub const Success: Self = Self(0);
    pub const ManifestDownloadFailure: Self = Self(1);
    pub const ManifestParseFailure: Self = Self(2);
    pub const UnsupportedManifestContentType: Self = Self(3);
    pub const UnsupportedManifestVersion: Self = Self(4);
    pub const UnsupportedManifestProfile: Self = Self(5);
    pub const UnknownFailure: Self = Self(6);
}
impl windows_core::imp::TypeKind for AdaptiveMediaSourceCreationStatus {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for AdaptiveMediaSourceCreationStatus {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.Media.Streaming.Adaptive.AdaptiveMediaSourceCreationStatus;i4)",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveMediaSourceDiagnostics(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AdaptiveMediaSourceDiagnostics,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for AdaptiveMediaSourceDiagnostics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAdaptiveMediaSourceDiagnostics>();
}
unsafe impl windows_core::Interface for AdaptiveMediaSourceDiagnostics {
    type Vtable = <IAdaptiveMediaSourceDiagnostics as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAdaptiveMediaSourceDiagnostics as windows_core::Interface>::IID;
}
impl core::ops::Deref for AdaptiveMediaSourceDiagnostics {
    type Target = IAdaptiveMediaSourceDiagnostics;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AdaptiveMediaSourceDiagnostics {
    const NAME: &'static str = "Windows.Media.Streaming.Adaptive.AdaptiveMediaSourceDiagnostics";
}
unsafe impl Send for AdaptiveMediaSourceDiagnostics {}
unsafe impl Sync for AdaptiveMediaSourceDiagnostics {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveMediaSourceDownloadBitrateChangedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AdaptiveMediaSourceDownloadBitrateChangedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for AdaptiveMediaSourceDownloadBitrateChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAdaptiveMediaSourceDownloadBitrateChangedEventArgs>();
}
unsafe impl windows_core::Interface for AdaptiveMediaSourceDownloadBitrateChangedEventArgs {
    type Vtable = <IAdaptiveMediaSourceDownloadBitrateChangedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IAdaptiveMediaSourceDownloadBitrateChangedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for AdaptiveMediaSourceDownloadBitrateChangedEventArgs {
    type Target = IAdaptiveMediaSourceDownloadBitrateChangedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AdaptiveMediaSourceDownloadBitrateChangedEventArgs {
    const NAME: &'static str = "Windows.Media.Streaming.Adaptive.AdaptiveMediaSourceDownloadBitrateChangedEventArgs";
}
unsafe impl Send for AdaptiveMediaSourceDownloadBitrateChangedEventArgs {}
unsafe impl Sync for AdaptiveMediaSourceDownloadBitrateChangedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveMediaSourceDownloadCompletedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AdaptiveMediaSourceDownloadCompletedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for AdaptiveMediaSourceDownloadCompletedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAdaptiveMediaSourceDownloadCompletedEventArgs>();
}
unsafe impl windows_core::Interface for AdaptiveMediaSourceDownloadCompletedEventArgs {
    type Vtable = <IAdaptiveMediaSourceDownloadCompletedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAdaptiveMediaSourceDownloadCompletedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for AdaptiveMediaSourceDownloadCompletedEventArgs {
    type Target = IAdaptiveMediaSourceDownloadCompletedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AdaptiveMediaSourceDownloadCompletedEventArgs {
    const NAME: &'static str = "Windows.Media.Streaming.Adaptive.AdaptiveMediaSourceDownloadCompletedEventArgs";
}
unsafe impl Send for AdaptiveMediaSourceDownloadCompletedEventArgs {}
unsafe impl Sync for AdaptiveMediaSourceDownloadCompletedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveMediaSourceDownloadFailedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AdaptiveMediaSourceDownloadFailedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for AdaptiveMediaSourceDownloadFailedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAdaptiveMediaSourceDownloadFailedEventArgs>();
}
unsafe impl windows_core::Interface for AdaptiveMediaSourceDownloadFailedEventArgs {
    type Vtable = <IAdaptiveMediaSourceDownloadFailedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAdaptiveMediaSourceDownloadFailedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for AdaptiveMediaSourceDownloadFailedEventArgs {
    type Target = IAdaptiveMediaSourceDownloadFailedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AdaptiveMediaSourceDownloadFailedEventArgs {
    const NAME: &'static str = "Windows.Media.Streaming.Adaptive.AdaptiveMediaSourceDownloadFailedEventArgs";
}
unsafe impl Send for AdaptiveMediaSourceDownloadFailedEventArgs {}
unsafe impl Sync for AdaptiveMediaSourceDownloadFailedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveMediaSourceDownloadRequestedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AdaptiveMediaSourceDownloadRequestedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for AdaptiveMediaSourceDownloadRequestedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAdaptiveMediaSourceDownloadRequestedEventArgs>();
}
unsafe impl windows_core::Interface for AdaptiveMediaSourceDownloadRequestedEventArgs {
    type Vtable = <IAdaptiveMediaSourceDownloadRequestedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAdaptiveMediaSourceDownloadRequestedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for AdaptiveMediaSourceDownloadRequestedEventArgs {
    type Target = IAdaptiveMediaSourceDownloadRequestedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AdaptiveMediaSourceDownloadRequestedEventArgs {
    const NAME: &'static str = "Windows.Media.Streaming.Adaptive.AdaptiveMediaSourceDownloadRequestedEventArgs";
}
unsafe impl Send for AdaptiveMediaSourceDownloadRequestedEventArgs {}
unsafe impl Sync for AdaptiveMediaSourceDownloadRequestedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveMediaSourcePlaybackBitrateChangedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AdaptiveMediaSourcePlaybackBitrateChangedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for AdaptiveMediaSourcePlaybackBitrateChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAdaptiveMediaSourcePlaybackBitrateChangedEventArgs>();
}
unsafe impl windows_core::Interface for AdaptiveMediaSourcePlaybackBitrateChangedEventArgs {
    type Vtable = <IAdaptiveMediaSourcePlaybackBitrateChangedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IAdaptiveMediaSourcePlaybackBitrateChangedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for AdaptiveMediaSourcePlaybackBitrateChangedEventArgs {
    type Target = IAdaptiveMediaSourcePlaybackBitrateChangedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AdaptiveMediaSourcePlaybackBitrateChangedEventArgs {
    const NAME: &'static str = "Windows.Media.Streaming.Adaptive.AdaptiveMediaSourcePlaybackBitrateChangedEventArgs";
}
unsafe impl Send for AdaptiveMediaSourcePlaybackBitrateChangedEventArgs {}
unsafe impl Sync for AdaptiveMediaSourcePlaybackBitrateChangedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AdaptiveMediaSourceResourceType(pub i32);
impl AdaptiveMediaSourceResourceType {
    pub const Manifest: Self = Self(0);
    pub const InitializationSegment: Self = Self(1);
    pub const MediaSegment: Self = Self(2);
    pub const Key: Self = Self(3);
    pub const InitializationVector: Self = Self(4);
    pub const MediaSegmentIndex: Self = Self(5);
}
impl windows_core::imp::TypeKind for AdaptiveMediaSourceResourceType {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for AdaptiveMediaSourceResourceType {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.Media.Streaming.Adaptive.AdaptiveMediaSourceResourceType;i4)",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Application(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Application, windows_core::IUnknown, windows_core::IInspectable);
impl Application {
    pub fn compose<T>(compose: T) -> windows_core::Result<Self>
    where
        T: windows_core::Compose,
    {
        Self::IApplicationFactory(|this| unsafe {
            let (derived__, base__) = windows_core::Compose::compose(compose);
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInstance)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(&derived__),
                base__ as *mut _ as _,
                &mut result__,
            )
            .ok()?;
            let _ = &derived__;
            windows_core::imp::Type::from_abi(result__)
        })
    }
    pub fn Current() -> windows_core::Result<Self> {
        Self::IApplicationStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Current)(windows_core::Interface::as_raw(this), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn Start<P0>(callback: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ApplicationInitializationCallback>,
    {
        Self::IApplicationStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).Start)(windows_core::Interface::as_raw(this), callback.param().abi())
                .ok()
        })
    }
    pub fn LoadComponent<P0, P1>(component: P0, resourcelocator: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
        P1: windows_core::Param<Uri>,
    {
        Self::IApplicationStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).LoadComponent)(
                windows_core::Interface::as_raw(this),
                component.param().abi(),
                resourcelocator.param().abi(),
            )
            .ok()
        })
    }
    pub fn LoadComponentWithResourceLocation<P0, P1>(
        component: P0,
        resourcelocator: P1,
        componentresourcelocation: ComponentResourceLocation,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
        P1: windows_core::Param<Uri>,
    {
        Self::IApplicationStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).LoadComponentWithResourceLocation)(
                windows_core::Interface::as_raw(this),
                component.param().abi(),
                resourcelocator.param().abi(),
                componentresourcelocation,
            )
            .ok()
        })
    }
    fn IApplicationFactory<R, F: FnOnce(&IApplicationFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<Application, IApplicationFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    fn IApplicationStatics<R, F: FnOnce(&IApplicationStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<Application, IApplicationStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for Application {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IApplication>();
}
unsafe impl windows_core::Interface for Application {
    type Vtable = <IApplication as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IApplication as windows_core::Interface>::IID;
}
impl core::ops::Deref for Application {
    type Target = IApplication;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Application {
    const NAME: &'static str = "Windows.UI.Xaml.Application";
}
unsafe impl Send for Application {}
unsafe impl Sync for Application {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationData(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ApplicationData, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ApplicationData, IClosable);
impl ApplicationData {
    pub fn Current() -> windows_core::Result<Self> {
        Self::IApplicationDataStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Current)(windows_core::Interface::as_raw(this), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IApplicationDataStatics<R, F: FnOnce(&IApplicationDataStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<ApplicationData, IApplicationDataStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for ApplicationData {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IApplicationData>();
}
unsafe impl windows_core::Interface for ApplicationData {
    type Vtable = <IApplicationData as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IApplicationData as windows_core::Interface>::IID;
}
impl core::ops::Deref for ApplicationData {
    type Target = IApplicationData;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ApplicationData {
    const NAME: &'static str = "Windows.Storage.ApplicationData";
}
unsafe impl Send for ApplicationData {}
unsafe impl Sync for ApplicationData {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationDataContainer(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ApplicationDataContainer, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for ApplicationDataContainer {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IApplicationDataContainer>();
}
unsafe impl windows_core::Interface for ApplicationDataContainer {
    type Vtable = <IApplicationDataContainer as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IApplicationDataContainer as windows_core::Interface>::IID;
}
impl core::ops::Deref for ApplicationDataContainer {
    type Target = IApplicationDataContainer;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ApplicationDataContainer {
    const NAME: &'static str = "Windows.Storage.ApplicationDataContainer";
}
unsafe impl Send for ApplicationDataContainer {}
unsafe impl Sync for ApplicationDataContainer {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ApplicationDataLocality(pub i32);
impl ApplicationDataLocality {
    pub const Local: Self = Self(0);
    pub const Roaming: Self = Self(1);
    pub const Temporary: Self = Self(2);
    pub const LocalCache: Self = Self(3);
    pub const SharedLocal: Self = Self(4);
}
impl windows_core::imp::TypeKind for ApplicationDataLocality {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for ApplicationDataLocality {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Storage.ApplicationDataLocality;i4)");
}
windows_core::imp::define_interface!(
    ApplicationDataSetVersionHandler,
    ApplicationDataSetVersionHandler_Vtbl,
    0xa05791e6_cc9f_4687_acab_a364fd785463
);
impl windows_core::RuntimeType for ApplicationDataSetVersionHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl ApplicationDataSetVersionHandler {
    pub fn new<F: Fn(windows_core::Ref<SetVersionRequest>) + 'static>(invoke: F) -> Self {
        let com =
            windows_core::imp::DelegateBox::<Self, F>::new(&ApplicationDataSetVersionHandlerBox::<F>::VTABLE, invoke);
        unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
    }
}
#[repr(C)]
pub struct ApplicationDataSetVersionHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        setversionrequest: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct ApplicationDataSetVersionHandlerBox<F: Fn(windows_core::Ref<SetVersionRequest>) + 'static>(
    core::marker::PhantomData<(fn() -> F,)>,
);
impl<F: Fn(windows_core::Ref<SetVersionRequest>) + 'static> ApplicationDataSetVersionHandlerBox<F> {
    const VTABLE: ApplicationDataSetVersionHandler_Vtbl = ApplicationDataSetVersionHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<ApplicationDataSetVersionHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<ApplicationDataSetVersionHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<ApplicationDataSetVersionHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        setversionrequest: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<ApplicationDataSetVersionHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&setversionrequest));
            windows_core::HRESULT(0)
        }
    }
}
windows_core::imp::define_interface!(
    ApplicationInitializationCallback,
    ApplicationInitializationCallback_Vtbl,
    0xb6351c55_c284_46e4_8310_fb0967fab76f
);
impl windows_core::RuntimeType for ApplicationInitializationCallback {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl ApplicationInitializationCallback {
    pub fn new<F: Fn(windows_core::Ref<ApplicationInitializationCallbackParams>) + 'static>(invoke: F) -> Self {
        let com =
            windows_core::imp::DelegateBox::<Self, F>::new(&ApplicationInitializationCallbackBox::<F>::VTABLE, invoke);
        unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
    }
}
#[repr(C)]
pub struct ApplicationInitializationCallback_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(this: *mut core::ffi::c_void, p: *mut core::ffi::c_void) -> windows_core::HRESULT,
}
struct ApplicationInitializationCallbackBox<F: Fn(windows_core::Ref<ApplicationInitializationCallbackParams>) + 'static>(
    core::marker::PhantomData<(fn() -> F,)>,
);
impl<F: Fn(windows_core::Ref<ApplicationInitializationCallbackParams>) + 'static>
    ApplicationInitializationCallbackBox<F>
{
    const VTABLE: ApplicationInitializationCallback_Vtbl = ApplicationInitializationCallback_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<ApplicationInitializationCallback, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<ApplicationInitializationCallback, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<ApplicationInitializationCallback, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(this: *mut core::ffi::c_void, p: *mut core::ffi::c_void) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<ApplicationInitializationCallback, F>);
            (this.invoke)(core::mem::transmute_copy(&p));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationInitializationCallbackParams(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ApplicationInitializationCallbackParams,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for ApplicationInitializationCallbackParams {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IApplicationInitializationCallbackParams>();
}
unsafe impl windows_core::Interface for ApplicationInitializationCallbackParams {
    type Vtable = <IApplicationInitializationCallbackParams as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IApplicationInitializationCallbackParams as windows_core::Interface>::IID;
}
impl core::ops::Deref for ApplicationInitializationCallbackParams {
    type Target = IApplicationInitializationCallbackParams;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ApplicationInitializationCallbackParams {
    const NAME: &'static str = "Windows.UI.Xaml.ApplicationInitializationCallbackParams";
}
unsafe impl Send for ApplicationInitializationCallbackParams {}
unsafe impl Sync for ApplicationInitializationCallbackParams {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioTrack(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(AudioTrack, windows_core::IUnknown, windows_core::IInspectable, IMediaTrack);
impl windows_core::RuntimeType for AudioTrack {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IMediaTrack>();
}
unsafe impl windows_core::Interface for AudioTrack {
    type Vtable = <IMediaTrack as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaTrack as windows_core::Interface>::IID;
}
impl core::ops::Deref for AudioTrack {
    type Target = IMediaTrack;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AudioTrack {
    const NAME: &'static str = "Windows.Media.Core.AudioTrack";
}
unsafe impl Send for AudioTrack {}
unsafe impl Sync for AudioTrack {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BasicProperties(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(BasicProperties, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for BasicProperties {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IBasicProperties>();
}
unsafe impl windows_core::Interface for BasicProperties {
    type Vtable = <IBasicProperties as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IBasicProperties as windows_core::Interface>::IID;
}
impl core::ops::Deref for BasicProperties {
    type Target = IBasicProperties;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for BasicProperties {
    const NAME: &'static str = "Windows.Storage.FileProperties.BasicProperties";
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingBase(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(BindingBase, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(BindingBase, DependencyObject);
impl windows_core::RuntimeType for BindingBase {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IBindingBase>();
}
unsafe impl windows_core::Interface for BindingBase {
    type Vtable = <IBindingBase as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IBindingBase as windows_core::Interface>::IID;
}
impl core::ops::Deref for BindingBase {
    type Target = IBindingBase;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for BindingBase {
    const NAME: &'static str = "Windows.UI.Xaml.Data.BindingBase";
}
unsafe impl Send for BindingBase {}
unsafe impl Sync for BindingBase {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BitmapCreateOptions(pub u32);
impl BitmapCreateOptions {
    pub const None: Self = Self(0);
    pub const IgnoreImageCache: Self = Self(8);
}
impl windows_core::imp::TypeKind for BitmapCreateOptions {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for BitmapCreateOptions {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Media.Imaging.BitmapCreateOptions;u4)");
}
impl BitmapCreateOptions {
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
impl core::ops::BitOr for BitmapCreateOptions {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl core::ops::BitAnd for BitmapCreateOptions {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
impl core::ops::BitOrAssign for BitmapCreateOptions {
    fn bitor_assign(&mut self, other: Self) {
        self.0.bitor_assign(other.0);
    }
}
impl core::ops::BitAndAssign for BitmapCreateOptions {
    fn bitand_assign(&mut self, other: Self) {
        self.0.bitand_assign(other.0);
    }
}
impl core::ops::Not for BitmapCreateOptions {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.not())
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitmapImage(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(BitmapImage, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(BitmapImage, BitmapSource, ImageSource, DependencyObject);
impl BitmapImage {
    pub fn CreateInstanceWithUriSource<P0>(urisource: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<Uri>,
    {
        Self::IBitmapImageFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInstanceWithUriSource)(
                windows_core::Interface::as_raw(this),
                urisource.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IBitmapImageFactory<R, F: FnOnce(&IBitmapImageFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<BitmapImage, IBitmapImageFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for BitmapImage {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IBitmapImage>();
}
unsafe impl windows_core::Interface for BitmapImage {
    type Vtable = <IBitmapImage as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IBitmapImage as windows_core::Interface>::IID;
}
impl core::ops::Deref for BitmapImage {
    type Target = IBitmapImage;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for BitmapImage {
    const NAME: &'static str = "Windows.UI.Xaml.Media.Imaging.BitmapImage";
}
unsafe impl Send for BitmapImage {}
unsafe impl Sync for BitmapImage {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitmapSource(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(BitmapSource, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(BitmapSource, ImageSource, DependencyObject);
impl windows_core::RuntimeType for BitmapSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IBitmapSource>();
}
unsafe impl windows_core::Interface for BitmapSource {
    type Vtable = <IBitmapSource as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IBitmapSource as windows_core::Interface>::IID;
}
impl core::ops::Deref for BitmapSource {
    type Target = IBitmapSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for BitmapSource {
    const NAME: &'static str = "Windows.UI.Xaml.Media.Imaging.BitmapSource";
}
unsafe impl Send for BitmapSource {}
unsafe impl Sync for BitmapSource {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Brush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Brush, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Brush, DependencyObject);
impl windows_core::RuntimeType for Brush {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IBrush>();
}
unsafe impl windows_core::Interface for Brush {
    type Vtable = <IBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IBrush as windows_core::Interface>::IID;
}
impl core::ops::Deref for Brush {
    type Target = IBrush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Brush {
    const NAME: &'static str = "Windows.UI.Xaml.Media.Brush";
}
unsafe impl Send for Brush {}
unsafe impl Sync for Brush {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ByteOrder(pub i32);
impl ByteOrder {
    pub const LittleEndian: Self = Self(0);
    pub const BigEndian: Self = Self(1);
}
impl windows_core::imp::TypeKind for ByteOrder {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for ByteOrder {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Storage.Streams.ByteOrder;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheMode(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(CacheMode, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(CacheMode, DependencyObject);
impl windows_core::RuntimeType for CacheMode {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ICacheMode>();
}
unsafe impl windows_core::Interface for CacheMode {
    type Vtable = <ICacheMode as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICacheMode as windows_core::Interface>::IID;
}
impl core::ops::Deref for CacheMode {
    type Target = ICacheMode;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for CacheMode {
    const NAME: &'static str = "Windows.UI.Xaml.Media.CacheMode";
}
unsafe impl Send for CacheMode {}
unsafe impl Sync for CacheMode {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedFileUpdaterActivatedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CachedFileUpdaterActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable,
    ICachedFileUpdaterActivatedEventArgs
);
impl windows_core::RuntimeType for CachedFileUpdaterActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICachedFileUpdaterActivatedEventArgs>();
}
unsafe impl windows_core::Interface for CachedFileUpdaterActivatedEventArgs {
    type Vtable = <ICachedFileUpdaterActivatedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICachedFileUpdaterActivatedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for CachedFileUpdaterActivatedEventArgs {
    type Target = ICachedFileUpdaterActivatedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for CachedFileUpdaterActivatedEventArgs {
    const NAME: &'static str = "Windows.ApplicationModel.Activation.CachedFileUpdaterActivatedEventArgs";
}
unsafe impl Send for CachedFileUpdaterActivatedEventArgs {}
unsafe impl Sync for CachedFileUpdaterActivatedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Canvas(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Canvas, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Canvas, Panel, FrameworkElement, UIElement, DependencyObject);
impl Canvas {
    pub fn LeftProperty() -> windows_core::Result<DependencyProperty> {
        Self::ICanvasStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).LeftProperty)(windows_core::Interface::as_raw(this), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn GetLeft<P0>(element: P0) -> windows_core::Result<f64>
    where
        P0: windows_core::Param<UIElement>,
    {
        Self::ICanvasStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetLeft)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub fn SetLeft<P0>(element: P0, length: f64) -> windows_core::Result<()>
    where
        P0: windows_core::Param<UIElement>,
    {
        Self::ICanvasStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).SetLeft)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                length,
            )
            .ok()
        })
    }
    pub fn TopProperty() -> windows_core::Result<DependencyProperty> {
        Self::ICanvasStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TopProperty)(windows_core::Interface::as_raw(this), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn GetTop<P0>(element: P0) -> windows_core::Result<f64>
    where
        P0: windows_core::Param<UIElement>,
    {
        Self::ICanvasStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetTop)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub fn SetTop<P0>(element: P0, length: f64) -> windows_core::Result<()>
    where
        P0: windows_core::Param<UIElement>,
    {
        Self::ICanvasStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).SetTop)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                length,
            )
            .ok()
        })
    }
    pub fn ZIndexProperty() -> windows_core::Result<DependencyProperty> {
        Self::ICanvasStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ZIndexProperty)(windows_core::Interface::as_raw(this), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn GetZIndex<P0>(element: P0) -> windows_core::Result<i32>
    where
        P0: windows_core::Param<UIElement>,
    {
        Self::ICanvasStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetZIndex)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub fn SetZIndex<P0>(element: P0, value: i32) -> windows_core::Result<()>
    where
        P0: windows_core::Param<UIElement>,
    {
        Self::ICanvasStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).SetZIndex)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                value,
            )
            .ok()
        })
    }
    fn ICanvasStatics<R, F: FnOnce(&ICanvasStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<Canvas, ICanvasStatics> = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for Canvas {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ICanvas>();
}
unsafe impl windows_core::Interface for Canvas {
    type Vtable = <ICanvas as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICanvas as windows_core::Interface>::IID;
}
impl core::ops::Deref for Canvas {
    type Target = ICanvas;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Canvas {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.Canvas";
}
unsafe impl Send for Canvas {}
unsafe impl Sync for Canvas {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CastingSource(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(CastingSource, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for CastingSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICastingSource>();
}
unsafe impl windows_core::Interface for CastingSource {
    type Vtable = <ICastingSource as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICastingSource as windows_core::Interface>::IID;
}
impl core::ops::Deref for CastingSource {
    type Target = ICastingSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for CastingSource {
    const NAME: &'static str = "Windows.Media.Casting.CastingSource";
}
unsafe impl Send for CastingSource {}
unsafe impl Sync for CastingSource {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ClickMode(pub i32);
impl ClickMode {
    pub const Release: Self = Self(0);
    pub const Press: Self = Self(1);
    pub const Hover: Self = Self(2);
}
impl windows_core::imp::TypeKind for ClickMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for ClickMode {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Controls.ClickMode;i4)");
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl windows_core::imp::TypeKind for Color {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Color {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.UI.Color;u1;u1;u1;u1)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnDefinition(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ColumnDefinition, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ColumnDefinition, DependencyObject);
impl windows_core::RuntimeType for ColumnDefinition {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IColumnDefinition>();
}
unsafe impl windows_core::Interface for ColumnDefinition {
    type Vtable = <IColumnDefinition as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IColumnDefinition as windows_core::Interface>::IID;
}
impl core::ops::Deref for ColumnDefinition {
    type Target = IColumnDefinition;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ColumnDefinition {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ColumnDefinition";
}
unsafe impl Send for ColumnDefinition {}
unsafe impl Sync for ColumnDefinition {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnDefinitionCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ColumnDefinitionCollection,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IVector<ColumnDefinition>
);
impl windows_core::RuntimeType for ColumnDefinitionCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector<ColumnDefinition>>();
}
unsafe impl windows_core::Interface for ColumnDefinitionCollection {
    type Vtable = <IVector<ColumnDefinition> as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVector<ColumnDefinition> as windows_core::Interface>::IID;
}
impl core::ops::Deref for ColumnDefinitionCollection {
    type Target = IVector<ColumnDefinition>;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ColumnDefinitionCollection {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ColumnDefinitionCollection";
}
unsafe impl Send for ColumnDefinitionCollection {}
unsafe impl Sync for ColumnDefinitionCollection {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ComponentResourceLocation(pub i32);
impl ComponentResourceLocation {
    pub const Application: Self = Self(0);
    pub const Nested: Self = Self(1);
}
impl windows_core::imp::TypeKind for ComponentResourceLocation {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for ComponentResourceLocation {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Xaml.Controls.Primitives.ComponentResourceLocation;i4)",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionTarget(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(CompositionTarget, windows_core::IUnknown, windows_core::IInspectable);
impl CompositionTarget {
    pub fn Rendering<F>(handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let handler: EventHandler<windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<EventHandler<windows_core::IInspectable>, F>::new(
                &EventHandlerBox::<windows_core::IInspectable, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        Self::ICompositionTargetStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(this).Rendering)(
                windows_core::Interface::as_raw(this),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                this.clone(),
                token__,
                windows_core::Interface::vtable(this).RemoveRendering,
            ))
        })
    }
    pub fn SurfaceContentsLost<F>(handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let handler: EventHandler<windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<EventHandler<windows_core::IInspectable>, F>::new(
                &EventHandlerBox::<windows_core::IInspectable, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        Self::ICompositionTargetStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(this).SurfaceContentsLost)(
                windows_core::Interface::as_raw(this),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                this.clone(),
                token__,
                windows_core::Interface::vtable(this).RemoveSurfaceContentsLost,
            ))
        })
    }
    fn ICompositionTargetStatics<R, F: FnOnce(&ICompositionTargetStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<CompositionTarget, ICompositionTargetStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for CompositionTarget {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionTarget>();
}
unsafe impl windows_core::Interface for CompositionTarget {
    type Vtable = <ICompositionTarget as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionTarget as windows_core::Interface>::IID;
}
impl core::ops::Deref for CompositionTarget {
    type Target = ICompositionTarget;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for CompositionTarget {
    const NAME: &'static str = "Windows.UI.Xaml.Media.CompositionTarget";
}
unsafe impl Send for CompositionTarget {}
unsafe impl Sync for CompositionTarget {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerContentChangingEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ContainerContentChangingEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for ContainerContentChangingEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IContainerContentChangingEventArgs>();
}
unsafe impl windows_core::Interface for ContainerContentChangingEventArgs {
    type Vtable = <IContainerContentChangingEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IContainerContentChangingEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for ContainerContentChangingEventArgs {
    type Target = IContainerContentChangingEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ContainerContentChangingEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ContainerContentChangingEventArgs";
}
unsafe impl Send for ContainerContentChangingEventArgs {}
unsafe impl Sync for ContainerContentChangingEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentControl(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ContentControl, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ContentControl, Control, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for ContentControl {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IContentControl>();
}
unsafe impl windows_core::Interface for ContentControl {
    type Vtable = <IContentControl as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IContentControl as windows_core::Interface>::IID;
}
impl core::ops::Deref for ContentControl {
    type Target = IContentControl;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ContentControl {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ContentControl";
}
unsafe impl Send for ContentControl {}
unsafe impl Sync for ContentControl {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextMenuEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ContextMenuEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ContextMenuEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for ContextMenuEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IContextMenuEventArgs>();
}
unsafe impl windows_core::Interface for ContextMenuEventArgs {
    type Vtable = <IContextMenuEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IContextMenuEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for ContextMenuEventArgs {
    type Target = IContextMenuEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ContextMenuEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ContextMenuEventArgs";
}
unsafe impl Send for ContextMenuEventArgs {}
unsafe impl Sync for ContextMenuEventArgs {}
windows_core::imp::define_interface!(
    ContextMenuOpeningEventHandler,
    ContextMenuOpeningEventHandler_Vtbl,
    0xdf945151_745c_4446_b2fc_216d765847a0
);
impl windows_core::RuntimeType for ContextMenuOpeningEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ContextMenuOpeningEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct ContextMenuOpeningEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ContextMenuEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ContextMenuEventArgs>) + 'static>
    ContextMenuOpeningEventHandlerBox<F>
{
    const VTABLE: ContextMenuOpeningEventHandler_Vtbl = ContextMenuOpeningEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<ContextMenuOpeningEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<ContextMenuOpeningEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<ContextMenuOpeningEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<ContextMenuOpeningEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Control(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Control, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Control, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for Control {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IControl>();
}
unsafe impl windows_core::Interface for Control {
    type Vtable = <IControl as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IControl as windows_core::Interface>::IID;
}
impl core::ops::Deref for Control {
    type Target = IControl;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Control {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.Control";
}
unsafe impl Send for Control {}
unsafe impl Sync for Control {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlTemplate(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ControlTemplate, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ControlTemplate, FrameworkTemplate, DependencyObject);
impl windows_core::RuntimeType for ControlTemplate {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IControlTemplate>();
}
unsafe impl windows_core::Interface for ControlTemplate {
    type Vtable = <IControlTemplate as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IControlTemplate as windows_core::Interface>::IID;
}
impl core::ops::Deref for ControlTemplate {
    type Target = IControlTemplate;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ControlTemplate {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ControlTemplate";
}
unsafe impl Send for ControlTemplate {}
unsafe impl Sync for ControlTemplate {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreDispatcher(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(CoreDispatcher, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(CoreDispatcher, ICoreAcceleratorKeys);
impl windows_core::RuntimeType for CoreDispatcher {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICoreDispatcher>();
}
unsafe impl windows_core::Interface for CoreDispatcher {
    type Vtable = <ICoreDispatcher as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICoreDispatcher as windows_core::Interface>::IID;
}
impl core::ops::Deref for CoreDispatcher {
    type Target = ICoreDispatcher;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for CoreDispatcher {
    const NAME: &'static str = "Windows.UI.Core.CoreDispatcher";
}
unsafe impl Send for CoreDispatcher {}
unsafe impl Sync for CoreDispatcher {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoreDispatcherPriority(pub i32);
impl CoreDispatcherPriority {
    pub const Idle: Self = Self(-2);
    pub const Low: Self = Self(-1);
    pub const Normal: Self = Self(0);
    pub const High: Self = Self(1);
}
impl windows_core::imp::TypeKind for CoreDispatcherPriority {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for CoreDispatcherPriority {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Core.CoreDispatcherPriority;i4)");
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoreProcessEventsOption(pub i32);
impl CoreProcessEventsOption {
    pub const ProcessOneAndAllPending: Self = Self(0);
    pub const ProcessOneIfPresent: Self = Self(1);
    pub const ProcessUntilQuit: Self = Self(2);
    pub const ProcessAllIfPresent: Self = Self(3);
}
impl windows_core::imp::TypeKind for CoreProcessEventsOption {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for CoreProcessEventsOption {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Core.CoreProcessEventsOption;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreWindow(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(CoreWindow, windows_core::IUnknown, windows_core::IInspectable, ICoreWindow);
impl windows_core::RuntimeType for CoreWindow {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ICoreWindow>();
}
unsafe impl windows_core::Interface for CoreWindow {
    type Vtable = <ICoreWindow as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICoreWindow as windows_core::Interface>::IID;
}
impl core::ops::Deref for CoreWindow {
    type Target = ICoreWindow;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for CoreWindow {
    const NAME: &'static str = "Windows.UI.Core.CoreWindow";
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreWindowEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CoreWindowEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable,
    ICoreWindowEventArgs
);
impl windows_core::RuntimeType for CoreWindowEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICoreWindowEventArgs>();
}
unsafe impl windows_core::Interface for CoreWindowEventArgs {
    type Vtable = <ICoreWindowEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICoreWindowEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for CoreWindowEventArgs {
    type Target = ICoreWindowEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for CoreWindowEventArgs {
    const NAME: &'static str = "Windows.UI.Core.CoreWindowEventArgs";
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CreationCollisionOption(pub i32);
impl CreationCollisionOption {
    pub const GenerateUniqueName: Self = Self(0);
    pub const ReplaceExisting: Self = Self(1);
    pub const FailIfExists: Self = Self(2);
    pub const OpenIfExists: Self = Self(3);
}
impl windows_core::imp::TypeKind for CreationCollisionOption {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for CreationCollisionOption {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Storage.CreationCollisionOption;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataTemplate(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(DataTemplate, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(DataTemplate, FrameworkTemplate, DependencyObject);
impl windows_core::RuntimeType for DataTemplate {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDataTemplate>();
}
unsafe impl windows_core::Interface for DataTemplate {
    type Vtable = <IDataTemplate as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDataTemplate as windows_core::Interface>::IID;
}
impl core::ops::Deref for DataTemplate {
    type Target = IDataTemplate;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DataTemplate {
    const NAME: &'static str = "Windows.UI.Xaml.DataTemplate";
}
unsafe impl Send for DataTemplate {}
unsafe impl Sync for DataTemplate {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataTemplateSelector(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(DataTemplateSelector, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for DataTemplateSelector {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDataTemplateSelector>();
}
unsafe impl windows_core::Interface for DataTemplateSelector {
    type Vtable = <IDataTemplateSelector as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDataTemplateSelector as windows_core::Interface>::IID;
}
impl core::ops::Deref for DataTemplateSelector {
    type Target = IDataTemplateSelector;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DataTemplateSelector {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.DataTemplateSelector";
}
unsafe impl Send for DataTemplateSelector {}
unsafe impl Sync for DataTemplateSelector {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataWriter(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(DataWriter, windows_core::IUnknown, windows_core::IInspectable, IDataWriter);
windows_core::imp::required_hierarchy!(DataWriter, IClosable);
impl DataWriter {
    pub fn CreateDataWriter<P0>(outputstream: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IOutputStream>,
    {
        Self::IDataWriterFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateDataWriter)(
                windows_core::Interface::as_raw(this),
                outputstream.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IDataWriterFactory<R, F: FnOnce(&IDataWriterFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<DataWriter, IDataWriterFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for DataWriter {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IDataWriter>();
}
unsafe impl windows_core::Interface for DataWriter {
    type Vtable = <IDataWriter as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDataWriter as windows_core::Interface>::IID;
}
impl core::ops::Deref for DataWriter {
    type Target = IDataWriter;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DataWriter {
    const NAME: &'static str = "Windows.Storage.Streams.DataWriter";
}
unsafe impl Send for DataWriter {}
unsafe impl Sync for DataWriter {}
pub type DataWriterStoreOperation = windows_future::IAsyncOperation<u32>;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyObject(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(DependencyObject, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for DependencyObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDependencyObject>();
}
unsafe impl windows_core::Interface for DependencyObject {
    type Vtable = <IDependencyObject as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDependencyObject as windows_core::Interface>::IID;
}
impl core::ops::Deref for DependencyObject {
    type Target = IDependencyObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DependencyObject {
    const NAME: &'static str = "Windows.UI.Xaml.DependencyObject";
}
unsafe impl Send for DependencyObject {}
unsafe impl Sync for DependencyObject {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyProperty(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(DependencyProperty, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for DependencyProperty {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDependencyProperty>();
}
unsafe impl windows_core::Interface for DependencyProperty {
    type Vtable = <IDependencyProperty as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDependencyProperty as windows_core::Interface>::IID;
}
impl core::ops::Deref for DependencyProperty {
    type Target = IDependencyProperty;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DependencyProperty {
    const NAME: &'static str = "Windows.UI.Xaml.DependencyProperty";
}
unsafe impl Send for DependencyProperty {}
unsafe impl Sync for DependencyProperty {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyPropertyChangedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    DependencyPropertyChangedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for DependencyPropertyChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDependencyPropertyChangedEventArgs>();
}
unsafe impl windows_core::Interface for DependencyPropertyChangedEventArgs {
    type Vtable = <IDependencyPropertyChangedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDependencyPropertyChangedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for DependencyPropertyChangedEventArgs {
    type Target = IDependencyPropertyChangedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DependencyPropertyChangedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.DependencyPropertyChangedEventArgs";
}
unsafe impl Send for DependencyPropertyChangedEventArgs {}
unsafe impl Sync for DependencyPropertyChangedEventArgs {}
windows_core::imp::define_interface!(
    DependencyPropertyChangedEventHandler,
    DependencyPropertyChangedEventHandler_Vtbl,
    0x09223e5a_75be_4499_8180_1ddc005421c0
);
impl windows_core::RuntimeType for DependencyPropertyChangedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct DependencyPropertyChangedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct DependencyPropertyChangedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DependencyPropertyChangedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DependencyPropertyChangedEventArgs>) + 'static,
> DependencyPropertyChangedEventHandlerBox<F>
{
    const VTABLE: DependencyPropertyChangedEventHandler_Vtbl = DependencyPropertyChangedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<DependencyPropertyChangedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<DependencyPropertyChangedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<DependencyPropertyChangedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<DependencyPropertyChangedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceInformation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(DeviceInformation, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for DeviceInformation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDeviceInformation>();
}
unsafe impl windows_core::Interface for DeviceInformation {
    type Vtable = <IDeviceInformation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDeviceInformation as windows_core::Interface>::IID;
}
impl core::ops::Deref for DeviceInformation {
    type Target = IDeviceInformation;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DeviceInformation {
    const NAME: &'static str = "Windows.Devices.Enumeration.DeviceInformation";
}
unsafe impl Send for DeviceInformation {}
unsafe impl Sync for DeviceInformation {}
windows_core::imp::define_interface!(DispatchedHandler, DispatchedHandler_Vtbl, 0xd1f276c4_98d8_4636_bf49_eb79507548e9);
impl windows_core::RuntimeType for DispatchedHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl DispatchedHandler {
    pub fn new<F: Fn() + 'static>(invoke: F) -> Self {
        let com = windows_core::imp::DelegateBox::<Self, F>::new(&DispatchedHandlerBox::<F>::VTABLE, invoke);
        unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
    }
}
#[repr(C)]
pub struct DispatchedHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(this: *mut core::ffi::c_void) -> windows_core::HRESULT,
}
struct DispatchedHandlerBox<F: Fn() + 'static>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn() + 'static> DispatchedHandlerBox<F> {
    const VTABLE: DispatchedHandler_Vtbl = DispatchedHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<DispatchedHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<DispatchedHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<DispatchedHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(this: *mut core::ffi::c_void) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<DispatchedHandler, F>);
            (this.invoke)();
            windows_core::HRESULT(0)
        }
    }
}
windows_core::imp::define_interface!(
    DoubleTappedEventHandler,
    DoubleTappedEventHandler_Vtbl,
    0x3124d025_04a7_4d45_825e_8204a624dbf4
);
impl windows_core::RuntimeType for DoubleTappedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct DoubleTappedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct DoubleTappedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DoubleTappedRoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DoubleTappedRoutedEventArgs>) + 'static>
    DoubleTappedEventHandlerBox<F>
{
    const VTABLE: DoubleTappedEventHandler_Vtbl = DoubleTappedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<DoubleTappedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<DoubleTappedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<DoubleTappedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<DoubleTappedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoubleTappedRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    DoubleTappedRoutedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(DoubleTappedRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for DoubleTappedRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDoubleTappedRoutedEventArgs>();
}
unsafe impl windows_core::Interface for DoubleTappedRoutedEventArgs {
    type Vtable = <IDoubleTappedRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDoubleTappedRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for DoubleTappedRoutedEventArgs {
    type Target = IDoubleTappedRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DoubleTappedRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.DoubleTappedRoutedEventArgs";
}
unsafe impl Send for DoubleTappedRoutedEventArgs {}
unsafe impl Sync for DoubleTappedRoutedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DownloadProgressEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(DownloadProgressEventArgs, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for DownloadProgressEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDownloadProgressEventArgs>();
}
unsafe impl windows_core::Interface for DownloadProgressEventArgs {
    type Vtable = <IDownloadProgressEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDownloadProgressEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for DownloadProgressEventArgs {
    type Target = IDownloadProgressEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DownloadProgressEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Media.Imaging.DownloadProgressEventArgs";
}
unsafe impl Send for DownloadProgressEventArgs {}
unsafe impl Sync for DownloadProgressEventArgs {}
windows_core::imp::define_interface!(
    DownloadProgressEventHandler,
    DownloadProgressEventHandler_Vtbl,
    0x1abaee23_74ee_4cc7_99ba_b171e3cda61e
);
impl windows_core::RuntimeType for DownloadProgressEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct DownloadProgressEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct DownloadProgressEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DownloadProgressEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DownloadProgressEventArgs>) + 'static>
    DownloadProgressEventHandlerBox<F>
{
    const VTABLE: DownloadProgressEventHandler_Vtbl = DownloadProgressEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<DownloadProgressEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<DownloadProgressEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<DownloadProgressEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<DownloadProgressEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DragEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(DragEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(DragEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for DragEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDragEventArgs>();
}
unsafe impl windows_core::Interface for DragEventArgs {
    type Vtable = <IDragEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDragEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for DragEventArgs {
    type Target = IDragEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DragEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.DragEventArgs";
}
unsafe impl Send for DragEventArgs {}
unsafe impl Sync for DragEventArgs {}
windows_core::imp::define_interface!(DragEventHandler, DragEventHandler_Vtbl, 0x2ab1a205_1e73_4bcf_aabc_57b97e21961d);
impl windows_core::RuntimeType for DragEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct DragEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct DragEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DragEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DragEventArgs>) + 'static>
    DragEventHandlerBox<F>
{
    const VTABLE: DragEventHandler_Vtbl = DragEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<DragEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<DragEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<DragEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this =
                &mut *(this as *mut *mut core::ffi::c_void as *mut windows_core::imp::DelegateBox<DragEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DragItemsStartingEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(DragItemsStartingEventArgs, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for DragItemsStartingEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDragItemsStartingEventArgs>();
}
unsafe impl windows_core::Interface for DragItemsStartingEventArgs {
    type Vtable = <IDragItemsStartingEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDragItemsStartingEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for DragItemsStartingEventArgs {
    type Target = IDragItemsStartingEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DragItemsStartingEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.DragItemsStartingEventArgs";
}
unsafe impl Send for DragItemsStartingEventArgs {}
unsafe impl Sync for DragItemsStartingEventArgs {}
windows_core::imp::define_interface!(
    DragItemsStartingEventHandler,
    DragItemsStartingEventHandler_Vtbl,
    0x3aaeab4c_14cb_4434_becc_88a8585c2f89
);
impl windows_core::RuntimeType for DragItemsStartingEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct DragItemsStartingEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct DragItemsStartingEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DragItemsStartingEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DragItemsStartingEventArgs>) + 'static>
    DragItemsStartingEventHandlerBox<F>
{
    const VTABLE: DragItemsStartingEventHandler_Vtbl = DragItemsStartingEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<DragItemsStartingEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<DragItemsStartingEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<DragItemsStartingEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<DragItemsStartingEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventHandler<T>(windows_core::IUnknown, core::marker::PhantomData<T>)
where
    T: windows_core::RuntimeType + 'static;
unsafe impl<T: windows_core::RuntimeType + 'static> windows_core::Interface for EventHandler<T> {
    type Vtable = EventHandler_Vtbl<T>;
    const IID: windows_core::GUID = windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
}
impl<T: windows_core::RuntimeType + 'static> windows_core::RuntimeType for EventHandler<T> {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
        .push_slice(b"pinterface({9de1c535-6ae1-11e0-84e1-18a905bcc53f}")
        .push_slice(b";")
        .push_other(T::SIGNATURE)
        .push_slice(b")");
}
#[repr(C)]
pub struct EventHandler_Vtbl<T>
where
    T: windows_core::RuntimeType + 'static,
{
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        args: windows_core::imp::AbiType<T>,
    ) -> windows_core::HRESULT,
    T: core::marker::PhantomData<T>,
}
struct EventHandlerBox<T, F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<T>) + 'static>(
    core::marker::PhantomData<(T, fn() -> F)>,
)
where
    T: windows_core::RuntimeType + 'static;
impl<
    T: windows_core::RuntimeType + 'static,
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<T>) + 'static,
> EventHandlerBox<T, F>
{
    const VTABLE: EventHandler_Vtbl<T> = EventHandler_Vtbl::<T> {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<EventHandler<T>, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<EventHandler<T>, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<EventHandler<T>, F>::Release,
        },
        Invoke: Self::Invoke,
        T: core::marker::PhantomData::<T>,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        args: windows_core::imp::AbiType<T>,
    ) -> windows_core::HRESULT {
        unsafe {
            let this =
                &mut *(this as *mut *mut core::ffi::c_void as *mut windows_core::imp::DelegateBox<EventHandler<T>, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&args));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExceptionRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ExceptionRoutedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ExceptionRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for ExceptionRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IExceptionRoutedEventArgs>();
}
unsafe impl windows_core::Interface for ExceptionRoutedEventArgs {
    type Vtable = <IExceptionRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IExceptionRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for ExceptionRoutedEventArgs {
    type Target = IExceptionRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ExceptionRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.ExceptionRoutedEventArgs";
}
unsafe impl Send for ExceptionRoutedEventArgs {}
unsafe impl Sync for ExceptionRoutedEventArgs {}
windows_core::imp::define_interface!(
    ExceptionRoutedEventHandler,
    ExceptionRoutedEventHandler_Vtbl,
    0x68e0e810_f6ea_42bc_855b_5d9b67e6a262
);
impl windows_core::RuntimeType for ExceptionRoutedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ExceptionRoutedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct ExceptionRoutedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ExceptionRoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ExceptionRoutedEventArgs>) + 'static>
    ExceptionRoutedEventHandlerBox<F>
{
    const VTABLE: ExceptionRoutedEventHandler_Vtbl = ExceptionRoutedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<ExceptionRoutedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<ExceptionRoutedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<ExceptionRoutedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<ExceptionRoutedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileActivatedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    FileActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IFileActivatedEventArgs
);
impl windows_core::RuntimeType for FileActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IFileActivatedEventArgs>();
}
unsafe impl windows_core::Interface for FileActivatedEventArgs {
    type Vtable = <IFileActivatedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IFileActivatedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for FileActivatedEventArgs {
    type Target = IFileActivatedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for FileActivatedEventArgs {
    const NAME: &'static str = "Windows.ApplicationModel.Activation.FileActivatedEventArgs";
}
unsafe impl Send for FileActivatedEventArgs {}
unsafe impl Sync for FileActivatedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FileAttributes(pub u32);
impl FileAttributes {
    pub const Normal: Self = Self(0);
    pub const ReadOnly: Self = Self(1);
    pub const Directory: Self = Self(16);
    pub const Archive: Self = Self(32);
    pub const Temporary: Self = Self(256);
    pub const LocallyIncomplete: Self = Self(512);
}
impl windows_core::imp::TypeKind for FileAttributes {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for FileAttributes {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Storage.FileAttributes;u4)");
}
impl FileAttributes {
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
impl core::ops::BitOr for FileAttributes {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl core::ops::BitAnd for FileAttributes {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
impl core::ops::BitOrAssign for FileAttributes {
    fn bitor_assign(&mut self, other: Self) {
        self.0.bitor_assign(other.0);
    }
}
impl core::ops::BitAndAssign for FileAttributes {
    fn bitand_assign(&mut self, other: Self) {
        self.0.bitand_assign(other.0);
    }
}
impl core::ops::Not for FileAttributes {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.not())
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileOpenPickerActivatedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    FileOpenPickerActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IFileOpenPickerActivatedEventArgs
);
impl windows_core::RuntimeType for FileOpenPickerActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IFileOpenPickerActivatedEventArgs>();
}
unsafe impl windows_core::Interface for FileOpenPickerActivatedEventArgs {
    type Vtable = <IFileOpenPickerActivatedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IFileOpenPickerActivatedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for FileOpenPickerActivatedEventArgs {
    type Target = IFileOpenPickerActivatedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for FileOpenPickerActivatedEventArgs {
    const NAME: &'static str = "Windows.ApplicationModel.Activation.FileOpenPickerActivatedEventArgs";
}
unsafe impl Send for FileOpenPickerActivatedEventArgs {}
unsafe impl Sync for FileOpenPickerActivatedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileSavePickerActivatedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    FileSavePickerActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IFileSavePickerActivatedEventArgs
);
impl windows_core::RuntimeType for FileSavePickerActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IFileSavePickerActivatedEventArgs>();
}
unsafe impl windows_core::Interface for FileSavePickerActivatedEventArgs {
    type Vtable = <IFileSavePickerActivatedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IFileSavePickerActivatedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for FileSavePickerActivatedEventArgs {
    type Target = IFileSavePickerActivatedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for FileSavePickerActivatedEventArgs {
    const NAME: &'static str = "Windows.ApplicationModel.Activation.FileSavePickerActivatedEventArgs";
}
unsafe impl Send for FileSavePickerActivatedEventArgs {}
unsafe impl Sync for FileSavePickerActivatedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FlowDirection(pub i32);
impl FlowDirection {
    pub const LeftToRight: Self = Self(0);
    pub const RightToLeft: Self = Self(1);
}
impl windows_core::imp::TypeKind for FlowDirection {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for FlowDirection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.FlowDirection;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlyoutBase(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(FlyoutBase, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(FlyoutBase, DependencyObject);
impl windows_core::RuntimeType for FlyoutBase {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IFlyoutBase>();
}
unsafe impl windows_core::Interface for FlyoutBase {
    type Vtable = <IFlyoutBase as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IFlyoutBase as windows_core::Interface>::IID;
}
impl core::ops::Deref for FlyoutBase {
    type Target = IFlyoutBase;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for FlyoutBase {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.Primitives.FlyoutBase";
}
unsafe impl Send for FlyoutBase {}
unsafe impl Sync for FlyoutBase {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FlyoutPlacementMode(pub i32);
impl FlyoutPlacementMode {
    pub const Top: Self = Self(0);
    pub const Bottom: Self = Self(1);
    pub const Left: Self = Self(2);
    pub const Right: Self = Self(3);
    pub const Full: Self = Self(4);
    pub const TopEdgeAlignedLeft: Self = Self(5);
    pub const TopEdgeAlignedRight: Self = Self(6);
    pub const BottomEdgeAlignedLeft: Self = Self(7);
    pub const BottomEdgeAlignedRight: Self = Self(8);
    pub const LeftEdgeAlignedTop: Self = Self(9);
    pub const LeftEdgeAlignedBottom: Self = Self(10);
    pub const RightEdgeAlignedTop: Self = Self(11);
    pub const RightEdgeAlignedBottom: Self = Self(12);
    pub const Auto: Self = Self(13);
}
impl windows_core::imp::TypeKind for FlyoutPlacementMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for FlyoutPlacementMode {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Controls.Primitives.FlyoutPlacementMode;i4)");
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FocusState(pub i32);
impl FocusState {
    pub const Unfocused: Self = Self(0);
    pub const Pointer: Self = Self(1);
    pub const Keyboard: Self = Self(2);
    pub const Programmatic: Self = Self(3);
}
impl windows_core::imp::TypeKind for FocusState {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for FocusState {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.FocusState;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontFamily(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(FontFamily, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for FontFamily {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IFontFamily>();
}
unsafe impl windows_core::Interface for FontFamily {
    type Vtable = <IFontFamily as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IFontFamily as windows_core::Interface>::IID;
}
impl core::ops::Deref for FontFamily {
    type Target = IFontFamily;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for FontFamily {
    const NAME: &'static str = "Windows.UI.Xaml.Media.FontFamily";
}
unsafe impl Send for FontFamily {}
unsafe impl Sync for FontFamily {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontStretch(pub i32);
impl FontStretch {
    pub const Undefined: Self = Self(0);
    pub const UltraCondensed: Self = Self(1);
    pub const ExtraCondensed: Self = Self(2);
    pub const Condensed: Self = Self(3);
    pub const SemiCondensed: Self = Self(4);
    pub const Normal: Self = Self(5);
    pub const SemiExpanded: Self = Self(6);
    pub const Expanded: Self = Self(7);
    pub const ExtraExpanded: Self = Self(8);
    pub const UltraExpanded: Self = Self(9);
}
impl windows_core::imp::TypeKind for FontStretch {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for FontStretch {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Text.FontStretch;i4)");
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontStyle(pub i32);
impl FontStyle {
    pub const Normal: Self = Self(0);
    pub const Oblique: Self = Self(1);
    pub const Italic: Self = Self(2);
}
impl windows_core::imp::TypeKind for FontStyle {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for FontStyle {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Text.FontStyle;i4)");
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontWeight {
    pub weight: u16,
}
impl windows_core::imp::TypeKind for FontWeight {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for FontWeight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.UI.Text.FontWeight;u2)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameworkElement(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(FrameworkElement, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for FrameworkElement {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IFrameworkElement>();
}
unsafe impl windows_core::Interface for FrameworkElement {
    type Vtable = <IFrameworkElement as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IFrameworkElement as windows_core::Interface>::IID;
}
impl core::ops::Deref for FrameworkElement {
    type Target = IFrameworkElement;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for FrameworkElement {
    const NAME: &'static str = "Windows.UI.Xaml.FrameworkElement";
}
unsafe impl Send for FrameworkElement {}
unsafe impl Sync for FrameworkElement {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameworkTemplate(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(FrameworkTemplate, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(FrameworkTemplate, DependencyObject);
impl windows_core::RuntimeType for FrameworkTemplate {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IFrameworkTemplate>();
}
unsafe impl windows_core::Interface for FrameworkTemplate {
    type Vtable = <IFrameworkTemplate as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IFrameworkTemplate as windows_core::Interface>::IID;
}
impl core::ops::Deref for FrameworkTemplate {
    type Target = IFrameworkTemplate;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for FrameworkTemplate {
    const NAME: &'static str = "Windows.UI.Xaml.FrameworkTemplate";
}
unsafe impl Send for FrameworkTemplate {}
unsafe impl Sync for FrameworkTemplate {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneralTransform(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(GeneralTransform, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(GeneralTransform, DependencyObject);
impl windows_core::RuntimeType for GeneralTransform {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IGeneralTransform>();
}
unsafe impl windows_core::Interface for GeneralTransform {
    type Vtable = <IGeneralTransform as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IGeneralTransform as windows_core::Interface>::IID;
}
impl core::ops::Deref for GeneralTransform {
    type Target = IGeneralTransform;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for GeneralTransform {
    const NAME: &'static str = "Windows.UI.Xaml.Media.GeneralTransform";
}
unsafe impl Send for GeneralTransform {}
unsafe impl Sync for GeneralTransform {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Geometry(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Geometry, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Geometry, DependencyObject);
impl windows_core::RuntimeType for Geometry {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IGeometry>();
}
unsafe impl windows_core::Interface for Geometry {
    type Vtable = <IGeometry as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IGeometry as windows_core::Interface>::IID;
}
impl core::ops::Deref for Geometry {
    type Target = IGeometry;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Geometry {
    const NAME: &'static str = "Windows.UI.Xaml.Media.Geometry";
}
unsafe impl Send for Geometry {}
unsafe impl Sync for Geometry {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grid(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Grid, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Grid, Panel, FrameworkElement, UIElement, DependencyObject);
impl Grid {
    pub fn RowProperty() -> windows_core::Result<DependencyProperty> {
        Self::IGridStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RowProperty)(windows_core::Interface::as_raw(this), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn GetRow<P0>(element: P0) -> windows_core::Result<i32>
    where
        P0: windows_core::Param<FrameworkElement>,
    {
        Self::IGridStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetRow)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub fn SetRow<P0>(element: P0, value: i32) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FrameworkElement>,
    {
        Self::IGridStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).SetRow)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                value,
            )
            .ok()
        })
    }
    pub fn ColumnProperty() -> windows_core::Result<DependencyProperty> {
        Self::IGridStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ColumnProperty)(windows_core::Interface::as_raw(this), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn GetColumn<P0>(element: P0) -> windows_core::Result<i32>
    where
        P0: windows_core::Param<FrameworkElement>,
    {
        Self::IGridStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetColumn)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub fn SetColumn<P0>(element: P0, value: i32) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FrameworkElement>,
    {
        Self::IGridStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).SetColumn)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                value,
            )
            .ok()
        })
    }
    pub fn RowSpanProperty() -> windows_core::Result<DependencyProperty> {
        Self::IGridStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RowSpanProperty)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn GetRowSpan<P0>(element: P0) -> windows_core::Result<i32>
    where
        P0: windows_core::Param<FrameworkElement>,
    {
        Self::IGridStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetRowSpan)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub fn SetRowSpan<P0>(element: P0, value: i32) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FrameworkElement>,
    {
        Self::IGridStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).SetRowSpan)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                value,
            )
            .ok()
        })
    }
    pub fn ColumnSpanProperty() -> windows_core::Result<DependencyProperty> {
        Self::IGridStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ColumnSpanProperty)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn GetColumnSpan<P0>(element: P0) -> windows_core::Result<i32>
    where
        P0: windows_core::Param<FrameworkElement>,
    {
        Self::IGridStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetColumnSpan)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub fn SetColumnSpan<P0>(element: P0, value: i32) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FrameworkElement>,
    {
        Self::IGridStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).SetColumnSpan)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                value,
            )
            .ok()
        })
    }
    fn IGridStatics<R, F: FnOnce(&IGridStatics) -> windows_core::Result<R>>(callback: F) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<Grid, IGridStatics> = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for Grid {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IGrid>();
}
unsafe impl windows_core::Interface for Grid {
    type Vtable = <IGrid as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IGrid as windows_core::Interface>::IID;
}
impl core::ops::Deref for Grid {
    type Target = IGrid;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Grid {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.Grid";
}
unsafe impl Send for Grid {}
unsafe impl Sync for Grid {}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GridLength {
    pub value: f64,
    pub grid_unit_type: GridUnitType,
}
impl windows_core::imp::TypeKind for GridLength {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for GridLength {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"struct(Windows.UI.Xaml.GridLength;f8;enum(Windows.UI.Xaml.GridUnitType;i4))",
    );
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GridUnitType(pub i32);
impl GridUnitType {
    pub const Auto: Self = Self(0);
    pub const Pixel: Self = Self(1);
    pub const Star: Self = Self(2);
}
impl windows_core::imp::TypeKind for GridUnitType {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for GridUnitType {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.GridUnitType;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridView(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(GridView, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(
    GridView,
    ListViewBase,
    Selector,
    ItemsControl,
    Control,
    FrameworkElement,
    UIElement,
    DependencyObject
);
impl windows_core::RuntimeType for GridView {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IGridView>();
}
unsafe impl windows_core::Interface for GridView {
    type Vtable = <IGridView as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IGridView as windows_core::Interface>::IID;
}
impl core::ops::Deref for GridView {
    type Target = IGridView;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for GridView {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.GridView";
}
unsafe impl Send for GridView {}
unsafe impl Sync for GridView {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GroupHeaderPlacement(pub i32);
impl GroupHeaderPlacement {
    pub const Top: Self = Self(0);
    pub const Left: Self = Self(1);
}
impl windows_core::imp::TypeKind for GroupHeaderPlacement {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for GroupHeaderPlacement {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Xaml.Controls.Primitives.GroupHeaderPlacement;i4)",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupStyle(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(GroupStyle, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for GroupStyle {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IGroupStyle>();
}
unsafe impl windows_core::Interface for GroupStyle {
    type Vtable = <IGroupStyle as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IGroupStyle as windows_core::Interface>::IID;
}
impl core::ops::Deref for GroupStyle {
    type Target = IGroupStyle;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for GroupStyle {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.GroupStyle";
}
unsafe impl Send for GroupStyle {}
unsafe impl Sync for GroupStyle {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupStyleSelector(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(GroupStyleSelector, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for GroupStyleSelector {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IGroupStyleSelector>();
}
unsafe impl windows_core::Interface for GroupStyleSelector {
    type Vtable = <IGroupStyleSelector as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IGroupStyleSelector as windows_core::Interface>::IID;
}
impl core::ops::Deref for GroupStyleSelector {
    type Target = IGroupStyleSelector;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for GroupStyleSelector {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.GroupStyleSelector";
}
unsafe impl Send for GroupStyleSelector {}
unsafe impl Sync for GroupStyleSelector {}
windows_core::imp::define_interface!(
    HoldingEventHandler,
    HoldingEventHandler_Vtbl,
    0xecae8ccd_8e5e_4fbe_9846_30a6370afcdf
);
impl windows_core::RuntimeType for HoldingEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct HoldingEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct HoldingEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<HoldingRoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<HoldingRoutedEventArgs>) + 'static>
    HoldingEventHandlerBox<F>
{
    const VTABLE: HoldingEventHandler_Vtbl = HoldingEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<HoldingEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<HoldingEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<HoldingEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<HoldingEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HoldingRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(HoldingRoutedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(HoldingRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for HoldingRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHoldingRoutedEventArgs>();
}
unsafe impl windows_core::Interface for HoldingRoutedEventArgs {
    type Vtable = <IHoldingRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHoldingRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for HoldingRoutedEventArgs {
    type Target = IHoldingRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HoldingRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.HoldingRoutedEventArgs";
}
unsafe impl Send for HoldingRoutedEventArgs {}
unsafe impl Sync for HoldingRoutedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HorizontalAlignment(pub i32);
impl HorizontalAlignment {
    pub const Left: Self = Self(0);
    pub const Center: Self = Self(1);
    pub const Right: Self = Self(2);
    pub const Stretch: Self = Self(3);
}
impl windows_core::imp::TypeKind for HorizontalAlignment {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for HorizontalAlignment {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.HorizontalAlignment;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostName(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(HostName, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for HostName {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IHostName>();
}
unsafe impl windows_core::Interface for HostName {
    type Vtable = <IHostName as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHostName as windows_core::Interface>::IID;
}
impl core::ops::Deref for HostName {
    type Target = IHostName;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HostName {
    const NAME: &'static str = "Windows.Networking.HostName";
}
unsafe impl Send for HostName {}
unsafe impl Sync for HostName {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpCacheDirectiveHeaderValueCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpCacheDirectiveHeaderValueCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpCacheDirectiveHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpCacheDirectiveHeaderValueCollection>();
}
unsafe impl windows_core::Interface for HttpCacheDirectiveHeaderValueCollection {
    type Vtable = <IHttpCacheDirectiveHeaderValueCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpCacheDirectiveHeaderValueCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpCacheDirectiveHeaderValueCollection {
    type Target = IHttpCacheDirectiveHeaderValueCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpCacheDirectiveHeaderValueCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpCacheDirectiveHeaderValueCollection";
}
unsafe impl Send for HttpCacheDirectiveHeaderValueCollection {}
unsafe impl Sync for HttpCacheDirectiveHeaderValueCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpClient(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(HttpClient, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(HttpClient, IClosable);
impl HttpClient {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<R, F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<HttpClient, windows_core::imp::IGenericFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for HttpClient {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IHttpClient>();
}
unsafe impl windows_core::Interface for HttpClient {
    type Vtable = <IHttpClient as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpClient as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpClient {
    type Target = IHttpClient;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpClient {
    const NAME: &'static str = "Windows.Web.Http.HttpClient";
}
unsafe impl Send for HttpClient {}
unsafe impl Sync for HttpClient {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HttpCompletionOption(pub i32);
impl HttpCompletionOption {
    pub const ResponseContentRead: Self = Self(0);
    pub const ResponseHeadersRead: Self = Self(1);
}
impl windows_core::imp::TypeKind for HttpCompletionOption {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for HttpCompletionOption {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Web.Http.HttpCompletionOption;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpConnectionOptionHeaderValueCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpConnectionOptionHeaderValueCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpConnectionOptionHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpConnectionOptionHeaderValueCollection>();
}
unsafe impl windows_core::Interface for HttpConnectionOptionHeaderValueCollection {
    type Vtable = <IHttpConnectionOptionHeaderValueCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpConnectionOptionHeaderValueCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpConnectionOptionHeaderValueCollection {
    type Target = IHttpConnectionOptionHeaderValueCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpConnectionOptionHeaderValueCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpConnectionOptionHeaderValueCollection";
}
unsafe impl Send for HttpConnectionOptionHeaderValueCollection {}
unsafe impl Sync for HttpConnectionOptionHeaderValueCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpContentCodingWithQualityHeaderValueCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpContentCodingWithQualityHeaderValueCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpContentCodingWithQualityHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpContentCodingWithQualityHeaderValueCollection>();
}
unsafe impl windows_core::Interface for HttpContentCodingWithQualityHeaderValueCollection {
    type Vtable = <IHttpContentCodingWithQualityHeaderValueCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IHttpContentCodingWithQualityHeaderValueCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpContentCodingWithQualityHeaderValueCollection {
    type Target = IHttpContentCodingWithQualityHeaderValueCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpContentCodingWithQualityHeaderValueCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpContentCodingWithQualityHeaderValueCollection";
}
unsafe impl Send for HttpContentCodingWithQualityHeaderValueCollection {}
unsafe impl Sync for HttpContentCodingWithQualityHeaderValueCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpCookiePairHeaderValueCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpCookiePairHeaderValueCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpCookiePairHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpCookiePairHeaderValueCollection>();
}
unsafe impl windows_core::Interface for HttpCookiePairHeaderValueCollection {
    type Vtable = <IHttpCookiePairHeaderValueCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpCookiePairHeaderValueCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpCookiePairHeaderValueCollection {
    type Target = IHttpCookiePairHeaderValueCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpCookiePairHeaderValueCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpCookiePairHeaderValueCollection";
}
unsafe impl Send for HttpCookiePairHeaderValueCollection {}
unsafe impl Sync for HttpCookiePairHeaderValueCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpCredentialsHeaderValue(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(HttpCredentialsHeaderValue, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for HttpCredentialsHeaderValue {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpCredentialsHeaderValue>();
}
unsafe impl windows_core::Interface for HttpCredentialsHeaderValue {
    type Vtable = <IHttpCredentialsHeaderValue as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpCredentialsHeaderValue as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpCredentialsHeaderValue {
    type Target = IHttpCredentialsHeaderValue;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpCredentialsHeaderValue {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpCredentialsHeaderValue";
}
unsafe impl Send for HttpCredentialsHeaderValue {}
unsafe impl Sync for HttpCredentialsHeaderValue {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpExpectationHeaderValueCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpExpectationHeaderValueCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpExpectationHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpExpectationHeaderValueCollection>();
}
unsafe impl windows_core::Interface for HttpExpectationHeaderValueCollection {
    type Vtable = <IHttpExpectationHeaderValueCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpExpectationHeaderValueCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpExpectationHeaderValueCollection {
    type Target = IHttpExpectationHeaderValueCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpExpectationHeaderValueCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpExpectationHeaderValueCollection";
}
unsafe impl Send for HttpExpectationHeaderValueCollection {}
unsafe impl Sync for HttpExpectationHeaderValueCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpLanguageRangeWithQualityHeaderValueCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpLanguageRangeWithQualityHeaderValueCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpLanguageRangeWithQualityHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpLanguageRangeWithQualityHeaderValueCollection>();
}
unsafe impl windows_core::Interface for HttpLanguageRangeWithQualityHeaderValueCollection {
    type Vtable = <IHttpLanguageRangeWithQualityHeaderValueCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IHttpLanguageRangeWithQualityHeaderValueCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpLanguageRangeWithQualityHeaderValueCollection {
    type Target = IHttpLanguageRangeWithQualityHeaderValueCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpLanguageRangeWithQualityHeaderValueCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpLanguageRangeWithQualityHeaderValueCollection";
}
unsafe impl Send for HttpLanguageRangeWithQualityHeaderValueCollection {}
unsafe impl Sync for HttpLanguageRangeWithQualityHeaderValueCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpMediaTypeWithQualityHeaderValueCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpMediaTypeWithQualityHeaderValueCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpMediaTypeWithQualityHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpMediaTypeWithQualityHeaderValueCollection>();
}
unsafe impl windows_core::Interface for HttpMediaTypeWithQualityHeaderValueCollection {
    type Vtable = <IHttpMediaTypeWithQualityHeaderValueCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpMediaTypeWithQualityHeaderValueCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpMediaTypeWithQualityHeaderValueCollection {
    type Target = IHttpMediaTypeWithQualityHeaderValueCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpMediaTypeWithQualityHeaderValueCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpMediaTypeWithQualityHeaderValueCollection";
}
unsafe impl Send for HttpMediaTypeWithQualityHeaderValueCollection {}
unsafe impl Sync for HttpMediaTypeWithQualityHeaderValueCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpProductInfoHeaderValueCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpProductInfoHeaderValueCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpProductInfoHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpProductInfoHeaderValueCollection>();
}
unsafe impl windows_core::Interface for HttpProductInfoHeaderValueCollection {
    type Vtable = <IHttpProductInfoHeaderValueCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpProductInfoHeaderValueCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpProductInfoHeaderValueCollection {
    type Target = IHttpProductInfoHeaderValueCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpProductInfoHeaderValueCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpProductInfoHeaderValueCollection";
}
unsafe impl Send for HttpProductInfoHeaderValueCollection {}
unsafe impl Sync for HttpProductInfoHeaderValueCollection {}
#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HttpProgress {
    pub stage: HttpProgressStage,
    pub bytes_sent: u64,
    pub total_bytes_to_send: Option<windows_reference::IReference<u64>>,
    pub bytes_received: u64,
    pub total_bytes_to_receive: Option<windows_reference::IReference<u64>>,
    pub retries: u32,
}
impl windows_core::imp::TypeKind for HttpProgress {
    type TypeKind = windows_core::imp::CloneType;
}
impl windows_core::RuntimeType for HttpProgress {
    const SIGNATURE : windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice (b"struct(Windows.Web.Http.HttpProgress;enum(Windows.Web.Http.HttpProgressStage;i4);u8;pinterface({61c17706-2d65-11e0-9ae8-d48564015472};u8);u8;pinterface({61c17706-2d65-11e0-9ae8-d48564015472};u8);u4)") ;
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HttpProgressStage(pub i32);
impl HttpProgressStage {
    pub const None: Self = Self(0);
    pub const DetectingProxy: Self = Self(10);
    pub const ResolvingName: Self = Self(20);
    pub const ConnectingToServer: Self = Self(30);
    pub const NegotiatingSsl: Self = Self(40);
    pub const SendingHeaders: Self = Self(50);
    pub const SendingContent: Self = Self(60);
    pub const WaitingForResponse: Self = Self(70);
    pub const ReceivingHeaders: Self = Self(80);
    pub const ReceivingContent: Self = Self(90);
}
impl windows_core::imp::TypeKind for HttpProgressStage {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for HttpProgressStage {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Web.Http.HttpProgressStage;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpRequestHeaderCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpRequestHeaderCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpRequestHeaderCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpRequestHeaderCollection>();
}
unsafe impl windows_core::Interface for HttpRequestHeaderCollection {
    type Vtable = <IHttpRequestHeaderCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpRequestHeaderCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpRequestHeaderCollection {
    type Target = IHttpRequestHeaderCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpRequestHeaderCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpRequestHeaderCollection";
}
unsafe impl Send for HttpRequestHeaderCollection {}
unsafe impl Sync for HttpRequestHeaderCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpRequestMessage(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(HttpRequestMessage, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for HttpRequestMessage {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpRequestMessage>();
}
unsafe impl windows_core::Interface for HttpRequestMessage {
    type Vtable = <IHttpRequestMessage as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpRequestMessage as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpRequestMessage {
    type Target = IHttpRequestMessage;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpRequestMessage {
    const NAME: &'static str = "Windows.Web.Http.HttpRequestMessage";
}
unsafe impl Send for HttpRequestMessage {}
unsafe impl Sync for HttpRequestMessage {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponseHeaderCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpResponseHeaderCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpResponseHeaderCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpResponseHeaderCollection>();
}
unsafe impl windows_core::Interface for HttpResponseHeaderCollection {
    type Vtable = <IHttpResponseHeaderCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpResponseHeaderCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpResponseHeaderCollection {
    type Target = IHttpResponseHeaderCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpResponseHeaderCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpResponseHeaderCollection";
}
unsafe impl Send for HttpResponseHeaderCollection {}
unsafe impl Sync for HttpResponseHeaderCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponseMessage(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(HttpResponseMessage, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for HttpResponseMessage {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpResponseMessage>();
}
unsafe impl windows_core::Interface for HttpResponseMessage {
    type Vtable = <IHttpResponseMessage as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpResponseMessage as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpResponseMessage {
    type Target = IHttpResponseMessage;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpResponseMessage {
    const NAME: &'static str = "Windows.Web.Http.HttpResponseMessage";
}
unsafe impl Send for HttpResponseMessage {}
unsafe impl Sync for HttpResponseMessage {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HttpResponseMessageSource(pub i32);
impl HttpResponseMessageSource {
    pub const None: Self = Self(0);
    pub const Cache: Self = Self(1);
    pub const Network: Self = Self(2);
}
impl windows_core::imp::TypeKind for HttpResponseMessageSource {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for HttpResponseMessageSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Web.Http.HttpResponseMessageSource;i4)");
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HttpStatusCode(pub i32);
impl HttpStatusCode {
    pub const None: Self = Self(0);
    pub const Continue: Self = Self(100);
    pub const SwitchingProtocols: Self = Self(101);
    pub const Processing: Self = Self(102);
    pub const Ok: Self = Self(200);
    pub const Created: Self = Self(201);
    pub const Accepted: Self = Self(202);
    pub const NonAuthoritativeInformation: Self = Self(203);
    pub const NoContent: Self = Self(204);
    pub const ResetContent: Self = Self(205);
    pub const PartialContent: Self = Self(206);
    pub const MultiStatus: Self = Self(207);
    pub const AlreadyReported: Self = Self(208);
    pub const IMUsed: Self = Self(226);
    pub const MultipleChoices: Self = Self(300);
    pub const MovedPermanently: Self = Self(301);
    pub const Found: Self = Self(302);
    pub const SeeOther: Self = Self(303);
    pub const NotModified: Self = Self(304);
    pub const UseProxy: Self = Self(305);
    pub const TemporaryRedirect: Self = Self(307);
    pub const PermanentRedirect: Self = Self(308);
    pub const BadRequest: Self = Self(400);
    pub const Unauthorized: Self = Self(401);
    pub const PaymentRequired: Self = Self(402);
    pub const Forbidden: Self = Self(403);
    pub const NotFound: Self = Self(404);
    pub const MethodNotAllowed: Self = Self(405);
    pub const NotAcceptable: Self = Self(406);
    pub const ProxyAuthenticationRequired: Self = Self(407);
    pub const RequestTimeout: Self = Self(408);
    pub const Conflict: Self = Self(409);
    pub const Gone: Self = Self(410);
    pub const LengthRequired: Self = Self(411);
    pub const PreconditionFailed: Self = Self(412);
    pub const RequestEntityTooLarge: Self = Self(413);
    pub const RequestUriTooLong: Self = Self(414);
    pub const UnsupportedMediaType: Self = Self(415);
    pub const RequestedRangeNotSatisfiable: Self = Self(416);
    pub const ExpectationFailed: Self = Self(417);
    pub const UnprocessableEntity: Self = Self(422);
    pub const Locked: Self = Self(423);
    pub const FailedDependency: Self = Self(424);
    pub const UpgradeRequired: Self = Self(426);
    pub const PreconditionRequired: Self = Self(428);
    pub const TooManyRequests: Self = Self(429);
    pub const RequestHeaderFieldsTooLarge: Self = Self(431);
    pub const InternalServerError: Self = Self(500);
    pub const NotImplemented: Self = Self(501);
    pub const BadGateway: Self = Self(502);
    pub const ServiceUnavailable: Self = Self(503);
    pub const GatewayTimeout: Self = Self(504);
    pub const HttpVersionNotSupported: Self = Self(505);
    pub const VariantAlsoNegotiates: Self = Self(506);
    pub const InsufficientStorage: Self = Self(507);
    pub const LoopDetected: Self = Self(508);
    pub const NotExtended: Self = Self(510);
    pub const NetworkAuthenticationRequired: Self = Self(511);
}
impl windows_core::imp::TypeKind for HttpStatusCode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for HttpStatusCode {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Web.Http.HttpStatusCode;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpTransferCodingHeaderValueCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    HttpTransferCodingHeaderValueCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for HttpTransferCodingHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IHttpTransferCodingHeaderValueCollection>();
}
unsafe impl windows_core::Interface for HttpTransferCodingHeaderValueCollection {
    type Vtable = <IHttpTransferCodingHeaderValueCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IHttpTransferCodingHeaderValueCollection as windows_core::Interface>::IID;
}
impl core::ops::Deref for HttpTransferCodingHeaderValueCollection {
    type Target = IHttpTransferCodingHeaderValueCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for HttpTransferCodingHeaderValueCollection {
    const NAME: &'static str = "Windows.Web.Http.Headers.HttpTransferCodingHeaderValueCollection";
}
unsafe impl Send for HttpTransferCodingHeaderValueCollection {}
unsafe impl Sync for HttpTransferCodingHeaderValueCollection {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HttpVersion(pub i32);
impl HttpVersion {
    pub const None: Self = Self(0);
    pub const Http10: Self = Self(1);
    pub const Http11: Self = Self(2);
    pub const Http20: Self = Self(3);
}
impl windows_core::imp::TypeKind for HttpVersion {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for HttpVersion {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Web.Http.HttpVersion;i4)");
}
windows_core::imp::define_interface!(
    IActivatedEventArgs,
    IActivatedEventArgs_Vtbl,
    0xcf651713_cd08_4fd8_b697_a281b6544e2e
);
impl windows_core::RuntimeType for IActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IActivatedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct IActivatedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSource,
    IAdaptiveMediaSource_Vtbl,
    0x4c7332ef_d39f_4396_b4d9_043957a7c964
);
impl windows_core::RuntimeType for IAdaptiveMediaSource {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IAdaptiveMediaSource {
    pub fn IsLive(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsLive)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn DesiredLiveOffset(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DesiredLiveOffset)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetDesiredLiveOffset(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetDesiredLiveOffset)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn InitialBitrate(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).InitialBitrate)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetInitialBitrate(&self, value: u32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetInitialBitrate)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn CurrentDownloadBitrate(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentDownloadBitrate)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn CurrentPlaybackBitrate(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentPlaybackBitrate)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn AvailableBitrates(&self) -> windows_core::Result<windows_collections::IVectorView<u32>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AvailableBitrates)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn DesiredMinBitrate(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DesiredMinBitrate)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
            .and_then(|r__: windows_reference::IReference<u32>| r__.Value())
        }
    }
    pub fn SetDesiredMinBitrate(&self, value: Option<u32>) -> windows_core::Result<()> {
        let value__ = value.map(<windows_reference::IReference<u32> as From<_>>::from);
        unsafe {
            (windows_core::Interface::vtable(self).SetDesiredMinBitrate)(
                windows_core::Interface::as_raw(self),
                windows_core::Param::param(value__.as_ref()).abi(),
            )
            .ok()
        }
    }
    pub fn DesiredMaxBitrate(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DesiredMaxBitrate)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
            .and_then(|r__: windows_reference::IReference<u32>| r__.Value())
        }
    }
    pub fn SetDesiredMaxBitrate(&self, value: Option<u32>) -> windows_core::Result<()> {
        let value__ = value.map(<windows_reference::IReference<u32> as From<_>>::from);
        unsafe {
            (windows_core::Interface::vtable(self).SetDesiredMaxBitrate)(
                windows_core::Interface::as_raw(self),
                windows_core::Param::param(value__.as_ref()).abi(),
            )
            .ok()
        }
    }
    pub fn AudioOnlyPlayback(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AudioOnlyPlayback)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn InboundBitsPerSecond(&self) -> windows_core::Result<u64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).InboundBitsPerSecond)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn InboundBitsPerSecondWindow(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).InboundBitsPerSecondWindow)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetInboundBitsPerSecondWindow(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetInboundBitsPerSecondWindow)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn DownloadBitrateChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(
                windows_core::Ref<AdaptiveMediaSource>,
                windows_core::Ref<AdaptiveMediaSourceDownloadBitrateChangedEventArgs>,
            ) + 'static,
    {
        let handler: TypedEventHandler<AdaptiveMediaSource, AdaptiveMediaSourceDownloadBitrateChangedEventArgs> = {
            let com = windows_core::imp::DelegateBox::< TypedEventHandler < AdaptiveMediaSource , AdaptiveMediaSourceDownloadBitrateChangedEventArgs > , F >::new (& TypedEventHandlerBox::< AdaptiveMediaSource , AdaptiveMediaSourceDownloadBitrateChangedEventArgs , F >::VTABLE , handler) ;
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DownloadBitrateChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDownloadBitrateChanged,
            ))
        }
    }
    pub fn PlaybackBitrateChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(
                windows_core::Ref<AdaptiveMediaSource>,
                windows_core::Ref<AdaptiveMediaSourcePlaybackBitrateChangedEventArgs>,
            ) + 'static,
    {
        let handler: TypedEventHandler<AdaptiveMediaSource, AdaptiveMediaSourcePlaybackBitrateChangedEventArgs> = {
            let com = windows_core::imp::DelegateBox::< TypedEventHandler < AdaptiveMediaSource , AdaptiveMediaSourcePlaybackBitrateChangedEventArgs > , F >::new (& TypedEventHandlerBox::< AdaptiveMediaSource , AdaptiveMediaSourcePlaybackBitrateChangedEventArgs , F >::VTABLE , handler) ;
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PlaybackBitrateChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePlaybackBitrateChanged,
            ))
        }
    }
    pub fn DownloadRequested<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<AdaptiveMediaSource>, windows_core::Ref<AdaptiveMediaSourceDownloadRequestedEventArgs>)
            + 'static,
    {
        let handler: TypedEventHandler<AdaptiveMediaSource, AdaptiveMediaSourceDownloadRequestedEventArgs> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<AdaptiveMediaSource, AdaptiveMediaSourceDownloadRequestedEventArgs>,
                F,
            >::new(
                &TypedEventHandlerBox::<AdaptiveMediaSource, AdaptiveMediaSourceDownloadRequestedEventArgs, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DownloadRequested)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDownloadRequested,
            ))
        }
    }
    pub fn DownloadCompleted<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<AdaptiveMediaSource>, windows_core::Ref<AdaptiveMediaSourceDownloadCompletedEventArgs>)
            + 'static,
    {
        let handler: TypedEventHandler<AdaptiveMediaSource, AdaptiveMediaSourceDownloadCompletedEventArgs> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<AdaptiveMediaSource, AdaptiveMediaSourceDownloadCompletedEventArgs>,
                F,
            >::new(
                &TypedEventHandlerBox::<AdaptiveMediaSource, AdaptiveMediaSourceDownloadCompletedEventArgs, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DownloadCompleted)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDownloadCompleted,
            ))
        }
    }
    pub fn DownloadFailed<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<AdaptiveMediaSource>, windows_core::Ref<AdaptiveMediaSourceDownloadFailedEventArgs>)
            + 'static,
    {
        let handler: TypedEventHandler<AdaptiveMediaSource, AdaptiveMediaSourceDownloadFailedEventArgs> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<AdaptiveMediaSource, AdaptiveMediaSourceDownloadFailedEventArgs>,
                F,
            >::new(
                &TypedEventHandlerBox::<AdaptiveMediaSource, AdaptiveMediaSourceDownloadFailedEventArgs, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DownloadFailed)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDownloadFailed,
            ))
        }
    }
}
#[repr(C)]
pub struct IAdaptiveMediaSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub IsLive: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub DesiredLiveOffset:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::TimeSpan) -> windows_core::HRESULT,
    pub SetDesiredLiveOffset:
        unsafe extern "system" fn(*mut core::ffi::c_void, windows_time::TimeSpan) -> windows_core::HRESULT,
    pub InitialBitrate: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub SetInitialBitrate: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub CurrentDownloadBitrate: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub CurrentPlaybackBitrate: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub AvailableBitrates:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DesiredMinBitrate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetDesiredMinBitrate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DesiredMaxBitrate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetDesiredMaxBitrate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub AudioOnlyPlayback: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub InboundBitsPerSecond: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u64) -> windows_core::HRESULT,
    pub InboundBitsPerSecondWindow:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::TimeSpan) -> windows_core::HRESULT,
    pub SetInboundBitsPerSecondWindow:
        unsafe extern "system" fn(*mut core::ffi::c_void, windows_time::TimeSpan) -> windows_core::HRESULT,
    pub DownloadBitrateChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDownloadBitrateChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PlaybackBitrateChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePlaybackBitrateChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub DownloadRequested:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDownloadRequested: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub DownloadCompleted:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDownloadCompleted: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub DownloadFailed:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDownloadFailed: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSource3,
    IAdaptiveMediaSource3_Vtbl,
    0xba7023fd_c334_461b_a36e_c99f54f7174a
);
impl windows_core::RuntimeType for IAdaptiveMediaSource3 {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IAdaptiveMediaSource3 {
    pub fn MinLiveOffset(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MinLiveOffset)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
                .and_then(|r__: windows_reference::IReference<windows_time::TimeSpan>| r__.Value())
        }
    }
    pub fn MaxSeekableWindowSize(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MaxSeekableWindowSize)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
            .and_then(|r__: windows_reference::IReference<windows_time::TimeSpan>| r__.Value())
        }
    }
    pub fn DesiredSeekableWindowSize(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DesiredSeekableWindowSize)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
            .and_then(|r__: windows_reference::IReference<windows_time::TimeSpan>| r__.Value())
        }
    }
    pub fn SetDesiredSeekableWindowSize(&self, value: Option<windows_time::TimeSpan>) -> windows_core::Result<()> {
        let value__ = value.map(<windows_reference::IReference<windows_time::TimeSpan> as From<_>>::from);
        unsafe {
            (windows_core::Interface::vtable(self).SetDesiredSeekableWindowSize)(
                windows_core::Interface::as_raw(self),
                windows_core::Param::param(value__.as_ref()).abi(),
            )
            .ok()
        }
    }
    pub fn Diagnostics(&self) -> windows_core::Result<AdaptiveMediaSourceDiagnostics> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Diagnostics)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetCorrelatedTimes(&self) -> windows_core::Result<AdaptiveMediaSourceCorrelatedTimes> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCorrelatedTimes)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IAdaptiveMediaSource3_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub MinLiveOffset:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub MaxSeekableWindowSize:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DesiredSeekableWindowSize:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetDesiredSeekableWindowSize:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Diagnostics:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetCorrelatedTimes:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSourceCorrelatedTimes,
    IAdaptiveMediaSourceCorrelatedTimes_Vtbl,
    0x05108787_e032_48e1_ab8d_002b0b3051df
);
impl windows_core::RuntimeType for IAdaptiveMediaSourceCorrelatedTimes {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAdaptiveMediaSourceCorrelatedTimes_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSourceCreationResult,
    IAdaptiveMediaSourceCreationResult_Vtbl,
    0x4686b6b2_800f_4e31_9093_76d4782013e7
);
impl windows_core::RuntimeType for IAdaptiveMediaSourceCreationResult {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IAdaptiveMediaSourceCreationResult {
    pub fn Status(&self) -> windows_core::Result<AdaptiveMediaSourceCreationStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Status)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn MediaSource(&self) -> windows_core::Result<AdaptiveMediaSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MediaSource)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn HttpResponseMessage(&self) -> windows_core::Result<HttpResponseMessage> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).HttpResponseMessage)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IAdaptiveMediaSourceCreationResult_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Status: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut AdaptiveMediaSourceCreationStatus,
    ) -> windows_core::HRESULT,
    pub MediaSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub HttpResponseMessage:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSourceDiagnostics,
    IAdaptiveMediaSourceDiagnostics_Vtbl,
    0x9b24ee68_962e_448c_aebf_b29b56098e23
);
impl windows_core::RuntimeType for IAdaptiveMediaSourceDiagnostics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAdaptiveMediaSourceDiagnostics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSourceDownloadBitrateChangedEventArgs,
    IAdaptiveMediaSourceDownloadBitrateChangedEventArgs_Vtbl,
    0x670c0a44_e04e_4eff_816a_17399f78f4ba
);
impl windows_core::RuntimeType for IAdaptiveMediaSourceDownloadBitrateChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAdaptiveMediaSourceDownloadBitrateChangedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSourceDownloadCompletedEventArgs,
    IAdaptiveMediaSourceDownloadCompletedEventArgs_Vtbl,
    0x19240dc3_5b37_4a1a_8970_d621cb6ca83b
);
impl windows_core::RuntimeType for IAdaptiveMediaSourceDownloadCompletedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAdaptiveMediaSourceDownloadCompletedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSourceDownloadFailedEventArgs,
    IAdaptiveMediaSourceDownloadFailedEventArgs_Vtbl,
    0x37739048_f4ab_40a4_b135_c6dfd8bd7ff1
);
impl windows_core::RuntimeType for IAdaptiveMediaSourceDownloadFailedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IAdaptiveMediaSourceDownloadFailedEventArgs {
    pub fn ResourceType(&self) -> windows_core::Result<AdaptiveMediaSourceResourceType> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ResourceType)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn ResourceUri(&self) -> windows_core::Result<Uri> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ResourceUri)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn ResourceByteRangeOffset(&self) -> windows_core::Result<u64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ResourceByteRangeOffset)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
            .and_then(|r__: windows_reference::IReference<u64>| r__.Value())
        }
    }
    pub fn ResourceByteRangeLength(&self) -> windows_core::Result<u64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ResourceByteRangeLength)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
            .and_then(|r__: windows_reference::IReference<u64>| r__.Value())
        }
    }
    pub fn HttpResponseMessage(&self) -> windows_core::Result<HttpResponseMessage> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).HttpResponseMessage)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IAdaptiveMediaSourceDownloadFailedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ResourceType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut AdaptiveMediaSourceResourceType,
    ) -> windows_core::HRESULT,
    pub ResourceUri:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ResourceByteRangeOffset:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ResourceByteRangeLength:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub HttpResponseMessage:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSourceDownloadRequestedEventArgs,
    IAdaptiveMediaSourceDownloadRequestedEventArgs_Vtbl,
    0xc83fdffd_44a9_47a2_bf96_03398b4bfaaf
);
impl windows_core::RuntimeType for IAdaptiveMediaSourceDownloadRequestedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAdaptiveMediaSourceDownloadRequestedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSourcePlaybackBitrateChangedEventArgs,
    IAdaptiveMediaSourcePlaybackBitrateChangedEventArgs_Vtbl,
    0x23a29f6d_7dda_4a51_87a9_6fa8c5b292be
);
impl windows_core::RuntimeType for IAdaptiveMediaSourcePlaybackBitrateChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAdaptiveMediaSourcePlaybackBitrateChangedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IAdaptiveMediaSourceStatics,
    IAdaptiveMediaSourceStatics_Vtbl,
    0x50a6bd5d_66ef_4cd3_9579_9e660507dc3f
);
impl windows_core::RuntimeType for IAdaptiveMediaSourceStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAdaptiveMediaSourceStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub IsContentTypeSupported:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub CreateFromUriAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFromUriWithDownloaderAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFromStreamAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFromStreamWithDownloaderAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IApplication, IApplication_Vtbl, 0x74b861a1_7487_46a9_9a6e_c78b512726c5);
impl windows_core::RuntimeType for IApplication {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IApplication_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IApplicationData, IApplicationData_Vtbl, 0xc3da6fb7_b744_4b45_b0b8_223a0938d0dc);
impl windows_core::RuntimeType for IApplicationData {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IApplicationData {
    pub fn Version(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Version)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetVersionAsync<P1>(&self, desiredversion: u32, handler: P1) -> windows_core::Result<IAsyncAction>
    where
        P1: windows_core::Param<ApplicationDataSetVersionHandler>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SetVersionAsync)(
                windows_core::Interface::as_raw(self),
                desiredversion,
                handler.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn ClearAllAsync(&self) -> windows_core::Result<IAsyncAction> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ClearAllAsync)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn ClearAsync(&self, locality: ApplicationDataLocality) -> windows_core::Result<IAsyncAction> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ClearAsync)(
                windows_core::Interface::as_raw(self),
                locality,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn LocalSettings(&self) -> windows_core::Result<ApplicationDataContainer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LocalSettings)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn RoamingSettings(&self) -> windows_core::Result<ApplicationDataContainer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RoamingSettings)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn LocalFolder(&self) -> windows_core::Result<StorageFolder> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LocalFolder)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn RoamingFolder(&self) -> windows_core::Result<StorageFolder> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RoamingFolder)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn TemporaryFolder(&self) -> windows_core::Result<StorageFolder> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TemporaryFolder)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn DataChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<ApplicationData>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let handler: TypedEventHandler<ApplicationData, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::< TypedEventHandler < ApplicationData , windows_core::IInspectable > , F >::new (& TypedEventHandlerBox::< ApplicationData , windows_core::IInspectable , F >::VTABLE , handler) ;
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DataChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDataChanged,
            ))
        }
    }
    pub fn SignalDataChanged(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SignalDataChanged)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn RoamingStorageQuota(&self) -> windows_core::Result<u64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RoamingStorageQuota)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IApplicationData_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Version: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub SetVersionAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ClearAllAsync:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ClearAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        ApplicationDataLocality,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub LocalSettings:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub RoamingSettings:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub LocalFolder:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub RoamingFolder:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub TemporaryFolder:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DataChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDataChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub SignalDataChanged: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub RoamingStorageQuota: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IApplicationDataContainer,
    IApplicationDataContainer_Vtbl,
    0xc5aefd1e_f467_40ba_8566_ab640a441e1d
);
impl windows_core::RuntimeType for IApplicationDataContainer {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IApplicationDataContainer_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IApplicationDataStatics,
    IApplicationDataStatics_Vtbl,
    0x5612147b_e843_45e3_94d8_06169e3c8e17
);
impl windows_core::RuntimeType for IApplicationDataStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IApplicationDataStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Current:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IApplicationFactory,
    IApplicationFactory_Vtbl,
    0x93bbe361_be5a_4ee3_b4a3_95118dc97a89
);
impl windows_core::RuntimeType for IApplicationFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IApplicationFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateInstance: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IApplicationInitializationCallbackParams,
    IApplicationInitializationCallbackParams_Vtbl,
    0x751b792e_5772_4488_8b87_f547faa64474
);
impl windows_core::RuntimeType for IApplicationInitializationCallbackParams {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IApplicationInitializationCallbackParams_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IApplicationOverrides,
    IApplicationOverrides_Vtbl,
    0x25f99ff7_9347_459a_9fac_b2d0e11c1a0f
);
impl windows_core::RuntimeType for IApplicationOverrides {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Xaml.IApplicationOverrides");
}
impl IApplicationOverrides {
    pub fn OnActivated<P0>(&self, args: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IActivatedEventArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).OnActivated)(
                windows_core::Interface::as_raw(self),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn OnLaunched<P0>(&self, args: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<LaunchActivatedEventArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).OnLaunched)(
                windows_core::Interface::as_raw(self),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn OnFileActivated<P0>(&self, args: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FileActivatedEventArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).OnFileActivated)(
                windows_core::Interface::as_raw(self),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn OnSearchActivated<P0>(&self, args: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<SearchActivatedEventArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).OnSearchActivated)(
                windows_core::Interface::as_raw(self),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn OnShareTargetActivated<P0>(&self, args: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ShareTargetActivatedEventArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).OnShareTargetActivated)(
                windows_core::Interface::as_raw(self),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn OnFileOpenPickerActivated<P0>(&self, args: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FileOpenPickerActivatedEventArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).OnFileOpenPickerActivated)(
                windows_core::Interface::as_raw(self),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn OnFileSavePickerActivated<P0>(&self, args: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FileSavePickerActivatedEventArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).OnFileSavePickerActivated)(
                windows_core::Interface::as_raw(self),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn OnCachedFileUpdaterActivated<P0>(&self, args: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CachedFileUpdaterActivatedEventArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).OnCachedFileUpdaterActivated)(
                windows_core::Interface::as_raw(self),
                args.param().abi(),
            )
            .ok()
        }
    }
    pub fn OnWindowCreated<P0>(&self, args: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<WindowCreatedEventArgs>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).OnWindowCreated)(
                windows_core::Interface::as_raw(self),
                args.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeName for IApplicationOverrides {
    const NAME: &'static str = "Windows.UI.Xaml.IApplicationOverrides";
}
pub trait IApplicationOverrides_Impl: windows_core::IUnknownImpl {
    fn OnActivated(&self, args: windows_core::Ref<IActivatedEventArgs>) -> windows_core::Result<()>;
    fn OnLaunched(&self, args: windows_core::Ref<LaunchActivatedEventArgs>) -> windows_core::Result<()>;
    fn OnFileActivated(&self, args: windows_core::Ref<FileActivatedEventArgs>) -> windows_core::Result<()>;
    fn OnSearchActivated(&self, args: windows_core::Ref<SearchActivatedEventArgs>) -> windows_core::Result<()>;
    fn OnShareTargetActivated(
        &self,
        args: windows_core::Ref<ShareTargetActivatedEventArgs>,
    ) -> windows_core::Result<()>;
    fn OnFileOpenPickerActivated(
        &self,
        args: windows_core::Ref<FileOpenPickerActivatedEventArgs>,
    ) -> windows_core::Result<()>;
    fn OnFileSavePickerActivated(
        &self,
        args: windows_core::Ref<FileSavePickerActivatedEventArgs>,
    ) -> windows_core::Result<()>;
    fn OnCachedFileUpdaterActivated(
        &self,
        args: windows_core::Ref<CachedFileUpdaterActivatedEventArgs>,
    ) -> windows_core::Result<()>;
    fn OnWindowCreated(&self, args: windows_core::Ref<WindowCreatedEventArgs>) -> windows_core::Result<()>;
}
impl IApplicationOverrides_Vtbl {
    pub const fn new<Identity: IApplicationOverrides_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn OnActivated<Identity: IApplicationOverrides_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IApplicationOverrides_Impl::OnActivated(this, core::mem::transmute_copy(&args)).into()
            }
        }
        unsafe extern "system" fn OnLaunched<Identity: IApplicationOverrides_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IApplicationOverrides_Impl::OnLaunched(this, core::mem::transmute_copy(&args)).into()
            }
        }
        unsafe extern "system" fn OnFileActivated<Identity: IApplicationOverrides_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IApplicationOverrides_Impl::OnFileActivated(this, core::mem::transmute_copy(&args)).into()
            }
        }
        unsafe extern "system" fn OnSearchActivated<Identity: IApplicationOverrides_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IApplicationOverrides_Impl::OnSearchActivated(this, core::mem::transmute_copy(&args)).into()
            }
        }
        unsafe extern "system" fn OnShareTargetActivated<Identity: IApplicationOverrides_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IApplicationOverrides_Impl::OnShareTargetActivated(this, core::mem::transmute_copy(&args)).into()
            }
        }
        unsafe extern "system" fn OnFileOpenPickerActivated<
            Identity: IApplicationOverrides_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IApplicationOverrides_Impl::OnFileOpenPickerActivated(this, core::mem::transmute_copy(&args)).into()
            }
        }
        unsafe extern "system" fn OnFileSavePickerActivated<
            Identity: IApplicationOverrides_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IApplicationOverrides_Impl::OnFileSavePickerActivated(this, core::mem::transmute_copy(&args)).into()
            }
        }
        unsafe extern "system" fn OnCachedFileUpdaterActivated<
            Identity: IApplicationOverrides_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IApplicationOverrides_Impl::OnCachedFileUpdaterActivated(this, core::mem::transmute_copy(&args)).into()
            }
        }
        unsafe extern "system" fn OnWindowCreated<Identity: IApplicationOverrides_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            args: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IApplicationOverrides_Impl::OnWindowCreated(this, core::mem::transmute_copy(&args)).into()
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IApplicationOverrides, OFFSET>(),
            OnActivated: OnActivated::<Identity, OFFSET>,
            OnLaunched: OnLaunched::<Identity, OFFSET>,
            OnFileActivated: OnFileActivated::<Identity, OFFSET>,
            OnSearchActivated: OnSearchActivated::<Identity, OFFSET>,
            OnShareTargetActivated: OnShareTargetActivated::<Identity, OFFSET>,
            OnFileOpenPickerActivated: OnFileOpenPickerActivated::<Identity, OFFSET>,
            OnFileSavePickerActivated: OnFileSavePickerActivated::<Identity, OFFSET>,
            OnCachedFileUpdaterActivated: OnCachedFileUpdaterActivated::<Identity, OFFSET>,
            OnWindowCreated: OnWindowCreated::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IApplicationOverrides as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct IApplicationOverrides_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub OnActivated: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub OnLaunched: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub OnFileActivated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub OnSearchActivated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub OnShareTargetActivated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub OnFileOpenPickerActivated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub OnFileSavePickerActivated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub OnCachedFileUpdaterActivated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub OnWindowCreated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IApplicationStatics,
    IApplicationStatics_Vtbl,
    0x06499997_f7b4_45fe_b763_7577d1d3cb4a
);
impl windows_core::RuntimeType for IApplicationStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IApplicationStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Current:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Start: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub LoadComponent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub LoadComponentWithResourceLocation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        ComponentResourceLocation,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IAsyncAction, IAsyncAction_Vtbl, 0x5a648006_843a_4da9_865b_9d26e5dfad7b);
impl windows_core::RuntimeType for IAsyncAction {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IAsyncAction, windows_core::IUnknown, windows_core::IInspectable);
impl IAsyncAction {
    pub fn SetCompleted<P0>(&self, handler: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_future::AsyncActionCompletedHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetCompleted)(
                windows_core::Interface::as_raw(self),
                handler.param().abi(),
            )
            .ok()
        }
    }
    pub fn Completed(&self) -> windows_core::Result<windows_future::AsyncActionCompletedHandler> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Completed)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetResults(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).GetResults)(windows_core::Interface::as_raw(self)).ok() }
    }
}
unsafe impl Send for IAsyncAction {}
unsafe impl Sync for IAsyncAction {}
#[repr(C)]
pub struct IAsyncAction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub SetCompleted:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Completed:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetResults: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IBasicProperties, IBasicProperties_Vtbl, 0xd05d55db_785e_4a66_be02_9beec58aea81);
impl windows_core::RuntimeType for IBasicProperties {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IBasicProperties_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IBindingBase, IBindingBase_Vtbl, 0x1589a2ab_3d15_49bc_a447_8a5448e58870);
impl windows_core::RuntimeType for IBindingBase {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IBindingBase_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IBitmapImage, IBitmapImage_Vtbl, 0x31af3271_e3b4_442d_a341_4c0226b2725b);
impl windows_core::RuntimeType for IBitmapImage {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IBitmapImage {
    pub fn CreateOptions(&self) -> windows_core::Result<BitmapCreateOptions> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateOptions)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetCreateOptions(&self, value: BitmapCreateOptions) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetCreateOptions)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn UriSource(&self) -> windows_core::Result<Uri> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).UriSource)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetUriSource<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Uri>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetUriSource)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn DecodePixelWidth(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DecodePixelWidth)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetDecodePixelWidth(&self, value: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetDecodePixelWidth)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn DecodePixelHeight(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DecodePixelHeight)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetDecodePixelHeight(&self, value: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetDecodePixelHeight)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn DownloadProgress<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DownloadProgressEventArgs>) + 'static,
    {
        let handler: DownloadProgressEventHandler = {
            let com = windows_core::imp::DelegateBox::<DownloadProgressEventHandler, F>::new(
                &DownloadProgressEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DownloadProgress)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDownloadProgress,
            ))
        }
    }
    pub fn ImageOpened<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
    {
        let handler: RoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RoutedEventHandler, F>::new(
                &RoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ImageOpened)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveImageOpened,
            ))
        }
    }
    pub fn ImageFailed<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ExceptionRoutedEventArgs>) + 'static,
    {
        let handler: ExceptionRoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<ExceptionRoutedEventHandler, F>::new(
                &ExceptionRoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ImageFailed)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveImageFailed,
            ))
        }
    }
}
#[repr(C)]
pub struct IBitmapImage_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateOptions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut BitmapCreateOptions) -> windows_core::HRESULT,
    pub SetCreateOptions:
        unsafe extern "system" fn(*mut core::ffi::c_void, BitmapCreateOptions) -> windows_core::HRESULT,
    pub UriSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetUriSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DecodePixelWidth: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetDecodePixelWidth: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub DecodePixelHeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetDecodePixelHeight: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub DownloadProgress:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDownloadProgress: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ImageOpened:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveImageOpened: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ImageFailed:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveImageFailed: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IBitmapImageFactory,
    IBitmapImageFactory_Vtbl,
    0xc9132978_4810_4e5e_8087_03671ee60d85
);
impl windows_core::RuntimeType for IBitmapImageFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IBitmapImageFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateInstanceWithUriSource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IBitmapSource, IBitmapSource_Vtbl, 0x23d86411_202f_41b2_8c5b_a8a3b333800b);
impl windows_core::RuntimeType for IBitmapSource {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IBitmapSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IBrush, IBrush_Vtbl, 0x8806a321_1e06_422c_a1cc_01696559e021);
impl windows_core::RuntimeType for IBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IBuffer, IBuffer_Vtbl, 0x905a0fe0_bc53_11df_8c49_001e4fc686da);
impl windows_core::RuntimeType for IBuffer {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IBuffer, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct IBuffer_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IButtonBase, IButtonBase_Vtbl, 0xfa002c1a_494e_46cf_91d4_e14a8d798674);
impl windows_core::RuntimeType for IButtonBase {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IButtonBase {
    pub fn ClickMode(&self) -> windows_core::Result<ClickMode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ClickMode)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetClickMode(&self, value: ClickMode) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetClickMode)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn IsPointerOver(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsPointerOver)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn IsPressed(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsPressed)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn Command(&self) -> windows_core::Result<ICommand> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Command)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetCommand<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICommand>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetCommand)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn CommandParameter(&self) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CommandParameter)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetCommandParameter<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetCommandParameter)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Click<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
    {
        let handler: RoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RoutedEventHandler, F>::new(
                &RoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Click)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveClick,
            ))
        }
    }
}
#[repr(C)]
pub struct IButtonBase_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ClickMode: unsafe extern "system" fn(*mut core::ffi::c_void, *mut ClickMode) -> windows_core::HRESULT,
    pub SetClickMode: unsafe extern "system" fn(*mut core::ffi::c_void, ClickMode) -> windows_core::HRESULT,
    pub IsPointerOver: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub IsPressed: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub Command:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetCommand: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CommandParameter:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetCommandParameter:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Click:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveClick: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IButtonWithFlyout, IButtonWithFlyout_Vtbl, 0x09108f87_df6c_4180_9b3a_e60845825811);
impl windows_core::RuntimeType for IButtonWithFlyout {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IButtonWithFlyout {
    pub fn Flyout(&self) -> windows_core::Result<FlyoutBase> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Flyout)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetFlyout<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FlyoutBase>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetFlyout)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IButtonWithFlyout_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Flyout: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetFlyout: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(ICacheMode, ICacheMode_Vtbl, 0x98dc8b11_c6f9_4dab_b838_5fd5ec8c7350);
impl windows_core::RuntimeType for ICacheMode {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ICacheMode_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICachedFileUpdaterActivatedEventArgs,
    ICachedFileUpdaterActivatedEventArgs_Vtbl,
    0xd06eb1c7_3805_4ecb_b757_6cf15e26fef3
);
impl windows_core::RuntimeType for ICachedFileUpdaterActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(
    ICachedFileUpdaterActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
#[repr(C)]
pub struct ICachedFileUpdaterActivatedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(ICanvas, ICanvas_Vtbl, 0x79190e19_cd38_4823_aeae_64a77132f519);
impl windows_core::RuntimeType for ICanvas {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ICanvas_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(ICanvasStatics, ICanvasStatics_Vtbl, 0x40ce5c46_2962_446f_aafb_4cdc486939c9);
impl windows_core::RuntimeType for ICanvasStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ICanvasStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub LeftProperty:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetLeft:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetLeft:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub TopProperty:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetTop:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetTop: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub ZIndexProperty:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetZIndex:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetZIndex:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, i32) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(ICastingSource, ICastingSource_Vtbl, 0xf429ea72_3467_47e6_a027_522923e9d727);
impl windows_core::RuntimeType for ICastingSource {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ICastingSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IClosable, IClosable_Vtbl, 0x30d5a829_7fa4_4026_83bb_d75bae4ea99e);
impl windows_core::RuntimeType for IClosable {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IClosable, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct IClosable_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IColumnDefinition, IColumnDefinition_Vtbl, 0xf7f1b229_f024_467f_970a_7e705615db7b);
impl windows_core::RuntimeType for IColumnDefinition {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IColumnDefinition {
    pub fn Width(&self) -> windows_core::Result<GridLength> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Width)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetWidth(&self, value: GridLength) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetWidth)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn MaxWidth(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MaxWidth)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetMaxWidth(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMaxWidth)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn MinWidth(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MinWidth)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetMinWidth(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMinWidth)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn ActualWidth(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ActualWidth)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IColumnDefinition_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Width: unsafe extern "system" fn(*mut core::ffi::c_void, *mut GridLength) -> windows_core::HRESULT,
    pub SetWidth: unsafe extern "system" fn(*mut core::ffi::c_void, GridLength) -> windows_core::HRESULT,
    pub MaxWidth: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetMaxWidth: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub MinWidth: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetMinWidth: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub ActualWidth: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(ICommand, ICommand_Vtbl, 0xe5af3542_ca67_4081_995b_709dd13792df);
impl windows_core::RuntimeType for ICommand {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(ICommand, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct ICommand_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionTarget,
    ICompositionTarget_Vtbl,
    0x26cfbff0_713c_4bec_8803_e101f7b14ed3
);
impl windows_core::RuntimeType for ICompositionTarget {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ICompositionTarget_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionTargetStatics,
    ICompositionTargetStatics_Vtbl,
    0x2b1af03d_1ed2_4b59_bd00_7594ee92832b
);
impl windows_core::RuntimeType for ICompositionTargetStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ICompositionTargetStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Rendering:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveRendering: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub SurfaceContentsLost:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveSurfaceContentsLost: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IContainerContentChangingEventArgs,
    IContainerContentChangingEventArgs_Vtbl,
    0x07deeaa7_018f_4dda_b399_fd4e13a31bea
);
impl windows_core::RuntimeType for IContainerContentChangingEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IContainerContentChangingEventArgs {
    pub fn ItemContainer(&self) -> windows_core::Result<SelectorItem> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemContainer)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn InRecycleQueue(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).InRecycleQueue)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn ItemIndex(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemIndex)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn Item(&self) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Item)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Phase(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Phase)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn Handled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Handled)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetHandled(&self, value: bool) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetHandled)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn RegisterUpdateCallback<P0>(&self, callback: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<TypedEventHandler<ListViewBase, ContainerContentChangingEventArgs>>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RegisterUpdateCallback)(
                windows_core::Interface::as_raw(self),
                callback.param().abi(),
            )
            .ok()
        }
    }
    pub fn RegisterUpdateCallbackWithPhase<P1>(&self, callbackphase: u32, callback: P1) -> windows_core::Result<()>
    where
        P1: windows_core::Param<TypedEventHandler<ListViewBase, ContainerContentChangingEventArgs>>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RegisterUpdateCallbackWithPhase)(
                windows_core::Interface::as_raw(self),
                callbackphase,
                callback.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IContainerContentChangingEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ItemContainer:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub InRecycleQueue: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub ItemIndex: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub Item: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Phase: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub Handled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetHandled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub RegisterUpdateCallback:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub RegisterUpdateCallbackWithPhase:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IContentControl, IContentControl_Vtbl, 0xa26dd1dc_cd44_435c_be94_01d6241c231c);
impl windows_core::RuntimeType for IContentControl {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IContentControl {
    pub fn Content(&self) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Content)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetContent<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetContent)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn ContentTemplate(&self) -> windows_core::Result<DataTemplate> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentTemplate)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetContentTemplate<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<DataTemplate>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetContentTemplate)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn ContentTemplateSelector(&self) -> windows_core::Result<DataTemplateSelector> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentTemplateSelector)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetContentTemplateSelector<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<DataTemplateSelector>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetContentTemplateSelector)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn ContentTransitions(&self) -> windows_core::Result<TransitionCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentTransitions)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetContentTransitions<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<TransitionCollection>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetContentTransitions)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IContentControl_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Content:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetContent: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ContentTemplate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetContentTemplate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ContentTemplateSelector:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetContentTemplateSelector:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ContentTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetContentTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IContentControl2, IContentControl2_Vtbl, 0x652c208c_8ccf_4399_bd3e_5a015aa1bc03);
impl windows_core::RuntimeType for IContentControl2 {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IContentControl2 {
    pub fn ContentTemplateRoot(&self) -> windows_core::Result<UIElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentTemplateRoot)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IContentControl2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ContentTemplateRoot:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IContextMenuEventArgs,
    IContextMenuEventArgs_Vtbl,
    0x1f7df263_c14b_4528_b6f0_637999d83cc6
);
impl windows_core::RuntimeType for IContextMenuEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IContextMenuEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IControl, IControl_Vtbl, 0xa8912263_2951_4f58_a9c5_5a134eaa7f07);
impl windows_core::RuntimeType for IControl {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IControl {
    pub fn FontSize(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FontSize)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetFontSize(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetFontSize)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn FontFamily(&self) -> windows_core::Result<FontFamily> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FontFamily)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetFontFamily<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FontFamily>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetFontFamily)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn FontWeight(&self) -> windows_core::Result<FontWeight> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FontWeight)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetFontWeight(&self, value: FontWeight) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetFontWeight)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn FontStyle(&self) -> windows_core::Result<FontStyle> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FontStyle)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetFontStyle(&self, value: FontStyle) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetFontStyle)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn FontStretch(&self) -> windows_core::Result<FontStretch> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FontStretch)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetFontStretch(&self, value: FontStretch) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetFontStretch)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn CharacterSpacing(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CharacterSpacing)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetCharacterSpacing(&self, value: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetCharacterSpacing)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn Foreground(&self) -> windows_core::Result<Brush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Foreground)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetForeground<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetForeground)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn IsTabStop(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsTabStop)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetIsTabStop(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsTabStop)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn IsEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsEnabled)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetIsEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsEnabled)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn TabIndex(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TabIndex)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetTabIndex(&self, value: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTabIndex)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn TabNavigation(&self) -> windows_core::Result<KeyboardNavigationMode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TabNavigation)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetTabNavigation(&self, value: KeyboardNavigationMode) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTabNavigation)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn Template(&self) -> windows_core::Result<ControlTemplate> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Template)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetTemplate<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ControlTemplate>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetTemplate)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Padding(&self) -> windows_core::Result<Thickness> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Padding)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetPadding(&self, value: Thickness) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetPadding)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn HorizontalContentAlignment(&self) -> windows_core::Result<HorizontalAlignment> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).HorizontalContentAlignment)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetHorizontalContentAlignment(&self, value: HorizontalAlignment) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetHorizontalContentAlignment)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn VerticalContentAlignment(&self) -> windows_core::Result<VerticalAlignment> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).VerticalContentAlignment)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetVerticalContentAlignment(&self, value: VerticalAlignment) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetVerticalContentAlignment)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn Background(&self) -> windows_core::Result<Brush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Background)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetBackground<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetBackground)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn BorderThickness(&self) -> windows_core::Result<Thickness> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BorderThickness)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetBorderThickness(&self, value: Thickness) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetBorderThickness)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn BorderBrush(&self) -> windows_core::Result<Brush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BorderBrush)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetBorderBrush<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetBorderBrush)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn FocusState(&self) -> windows_core::Result<FocusState> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FocusState)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn IsEnabledChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DependencyPropertyChangedEventArgs>)
            + 'static,
    {
        let handler: DependencyPropertyChangedEventHandler = {
            let com = windows_core::imp::DelegateBox::<DependencyPropertyChangedEventHandler, F>::new(
                &DependencyPropertyChangedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).IsEnabledChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveIsEnabledChanged,
            ))
        }
    }
    pub fn ApplyTemplate(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ApplyTemplate)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn Focus(&self, value: FocusState) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Focus)(windows_core::Interface::as_raw(self), value, &mut result__)
                .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IControl_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub FontSize: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetFontSize: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub FontFamily:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetFontFamily:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub FontWeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut FontWeight) -> windows_core::HRESULT,
    pub SetFontWeight: unsafe extern "system" fn(*mut core::ffi::c_void, FontWeight) -> windows_core::HRESULT,
    pub FontStyle: unsafe extern "system" fn(*mut core::ffi::c_void, *mut FontStyle) -> windows_core::HRESULT,
    pub SetFontStyle: unsafe extern "system" fn(*mut core::ffi::c_void, FontStyle) -> windows_core::HRESULT,
    pub FontStretch: unsafe extern "system" fn(*mut core::ffi::c_void, *mut FontStretch) -> windows_core::HRESULT,
    pub SetFontStretch: unsafe extern "system" fn(*mut core::ffi::c_void, FontStretch) -> windows_core::HRESULT,
    pub CharacterSpacing: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetCharacterSpacing: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub Foreground:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetForeground:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub IsTabStop: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsTabStop: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub IsEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub TabIndex: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetTabIndex: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub TabNavigation:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut KeyboardNavigationMode) -> windows_core::HRESULT,
    pub SetTabNavigation:
        unsafe extern "system" fn(*mut core::ffi::c_void, KeyboardNavigationMode) -> windows_core::HRESULT,
    pub Template:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetTemplate: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Padding: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Thickness) -> windows_core::HRESULT,
    pub SetPadding: unsafe extern "system" fn(*mut core::ffi::c_void, Thickness) -> windows_core::HRESULT,
    pub HorizontalContentAlignment:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut HorizontalAlignment) -> windows_core::HRESULT,
    pub SetHorizontalContentAlignment:
        unsafe extern "system" fn(*mut core::ffi::c_void, HorizontalAlignment) -> windows_core::HRESULT,
    pub VerticalContentAlignment:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut VerticalAlignment) -> windows_core::HRESULT,
    pub SetVerticalContentAlignment:
        unsafe extern "system" fn(*mut core::ffi::c_void, VerticalAlignment) -> windows_core::HRESULT,
    pub Background:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetBackground:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub BorderThickness: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Thickness) -> windows_core::HRESULT,
    pub SetBorderThickness: unsafe extern "system" fn(*mut core::ffi::c_void, Thickness) -> windows_core::HRESULT,
    pub BorderBrush:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetBorderBrush:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub FocusState: unsafe extern "system" fn(*mut core::ffi::c_void, *mut FocusState) -> windows_core::HRESULT,
    pub IsEnabledChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveIsEnabledChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ApplyTemplate: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub Focus: unsafe extern "system" fn(*mut core::ffi::c_void, FocusState, *mut bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IControlTemplate, IControlTemplate_Vtbl, 0xefd2418e_41e0_48bb_8b82_91eda1ba3fe2);
impl windows_core::RuntimeType for IControlTemplate {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IControlTemplate_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICoreAcceleratorKeys,
    ICoreAcceleratorKeys_Vtbl,
    0x9ffdf7f5_b8c9_4ef0_b7d2_1de626561fc8
);
impl windows_core::RuntimeType for ICoreAcceleratorKeys {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(ICoreAcceleratorKeys, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct ICoreAcceleratorKeys_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(ICoreDispatcher, ICoreDispatcher_Vtbl, 0x60db2fa8_b705_4fde_a7d6_ebbb1891d39e);
impl windows_core::RuntimeType for ICoreDispatcher {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl ICoreDispatcher {
    pub fn HasThreadAccess(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).HasThreadAccess)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ProcessEvents(&self, options: CoreProcessEventsOption) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).ProcessEvents)(windows_core::Interface::as_raw(self), options).ok()
        }
    }
    pub fn RunAsync<P1>(
        &self,
        priority: CoreDispatcherPriority,
        agilecallback: P1,
    ) -> windows_core::Result<IAsyncAction>
    where
        P1: windows_core::Param<DispatchedHandler>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RunAsync)(
                windows_core::Interface::as_raw(self),
                priority,
                agilecallback.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn RunIdleAsync<P0>(&self, agilecallback: P0) -> windows_core::Result<IAsyncAction>
    where
        P0: windows_core::Param<IdleDispatchedHandler>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RunIdleAsync)(
                windows_core::Interface::as_raw(self),
                agilecallback.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct ICoreDispatcher_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub HasThreadAccess: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub ProcessEvents:
        unsafe extern "system" fn(*mut core::ffi::c_void, CoreProcessEventsOption) -> windows_core::HRESULT,
    pub RunAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        CoreDispatcherPriority,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RunIdleAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(ICoreWindow, ICoreWindow_Vtbl, 0x79b9d5f2_879e_4b89_b798_79e47598030c);
impl windows_core::RuntimeType for ICoreWindow {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(ICoreWindow, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct ICoreWindow_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICoreWindowEventArgs,
    ICoreWindowEventArgs_Vtbl,
    0x272b1ef3_c633_4da5_a26c_c6d0f56b29da
);
impl windows_core::RuntimeType for ICoreWindowEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(ICoreWindowEventArgs, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct ICoreWindowEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICredentialFactory,
    ICredentialFactory_Vtbl,
    0x54ef13a1_bf26_47b5_97dd_de779b7cad58
);
impl windows_core::RuntimeType for ICredentialFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ICredentialFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreatePasswordCredential: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IDataTemplate, IDataTemplate_Vtbl, 0x9910aec7_8ab5_4118_9bc6_09f45a35073d);
impl windows_core::RuntimeType for IDataTemplate {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IDataTemplate {
    pub fn LoadContent(&self) -> windows_core::Result<DependencyObject> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LoadContent)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDataTemplate_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub LoadContent:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDataTemplateSelector,
    IDataTemplateSelector_Vtbl,
    0xa907d496_46a0_4cd7_8dbe_f9a581df60b1
);
impl windows_core::RuntimeType for IDataTemplateSelector {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDataTemplateSelector_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IDataWriter, IDataWriter_Vtbl, 0x64b89265_d341_4922_b38a_dd4af8808c4e);
impl windows_core::RuntimeType for IDataWriter {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IDataWriter, windows_core::IUnknown, windows_core::IInspectable);
impl IDataWriter {
    pub fn UnstoredBufferLength(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).UnstoredBufferLength)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn UnicodeEncoding(&self) -> windows_core::Result<UnicodeEncoding> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).UnicodeEncoding)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetUnicodeEncoding(&self, value: UnicodeEncoding) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetUnicodeEncoding)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn ByteOrder(&self) -> windows_core::Result<ByteOrder> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ByteOrder)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetByteOrder(&self, value: ByteOrder) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetByteOrder)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn WriteByte(&self, value: u8) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).WriteByte)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn WriteBytes(&self, value: &[u8]) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).WriteBytes)(
                windows_core::Interface::as_raw(self),
                value.len().try_into().unwrap(),
                value.as_ptr(),
            )
            .ok()
        }
    }
    pub fn WriteBuffer<P0>(&self, buffer: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IBuffer>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).WriteBuffer)(
                windows_core::Interface::as_raw(self),
                buffer.param().abi(),
            )
            .ok()
        }
    }
    pub fn WriteBufferRange<P0>(&self, buffer: P0, start: u32, count: u32) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IBuffer>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).WriteBufferRange)(
                windows_core::Interface::as_raw(self),
                buffer.param().abi(),
                start,
                count,
            )
            .ok()
        }
    }
    pub fn WriteBoolean(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).WriteBoolean)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn WriteGuid(&self, value: windows_core::GUID) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).WriteGuid)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn WriteInt16(&self, value: i16) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).WriteInt16)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn WriteInt32(&self, value: i32) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).WriteInt32)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn WriteInt64(&self, value: i64) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).WriteInt64)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn WriteUInt16(&self, value: u16) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).WriteUInt16)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn WriteUInt32(&self, value: u32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).WriteUInt32)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn WriteUInt64(&self, value: u64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).WriteUInt64)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn WriteSingle(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).WriteSingle)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn WriteDouble(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).WriteDouble)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn WriteDateTime(&self, value: windows_time::DateTime) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).WriteDateTime)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn WriteTimeSpan(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).WriteTimeSpan)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn WriteString(&self, value: &str) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).WriteString)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn MeasureString(&self, value: &str) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MeasureString)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn StoreAsync(&self) -> windows_core::Result<DataWriterStoreOperation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).StoreAsync)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn FlushAsync(&self) -> windows_core::Result<windows_future::IAsyncOperation<bool>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FlushAsync)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn DetachBuffer(&self) -> windows_core::Result<IBuffer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DetachBuffer)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn DetachStream(&self) -> windows_core::Result<IOutputStream> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DetachStream)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDataWriter_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub UnstoredBufferLength: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub UnicodeEncoding:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut UnicodeEncoding) -> windows_core::HRESULT,
    pub SetUnicodeEncoding: unsafe extern "system" fn(*mut core::ffi::c_void, UnicodeEncoding) -> windows_core::HRESULT,
    pub ByteOrder: unsafe extern "system" fn(*mut core::ffi::c_void, *mut ByteOrder) -> windows_core::HRESULT,
    pub SetByteOrder: unsafe extern "system" fn(*mut core::ffi::c_void, ByteOrder) -> windows_core::HRESULT,
    pub WriteByte: unsafe extern "system" fn(*mut core::ffi::c_void, u8) -> windows_core::HRESULT,
    pub WriteBytes: unsafe extern "system" fn(*mut core::ffi::c_void, u32, *const u8) -> windows_core::HRESULT,
    pub WriteBuffer: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub WriteBufferRange:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, u32, u32) -> windows_core::HRESULT,
    pub WriteBoolean: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub WriteGuid: unsafe extern "system" fn(*mut core::ffi::c_void, windows_core::GUID) -> windows_core::HRESULT,
    pub WriteInt16: unsafe extern "system" fn(*mut core::ffi::c_void, i16) -> windows_core::HRESULT,
    pub WriteInt32: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub WriteInt64: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub WriteUInt16: unsafe extern "system" fn(*mut core::ffi::c_void, u16) -> windows_core::HRESULT,
    pub WriteUInt32: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub WriteUInt64: unsafe extern "system" fn(*mut core::ffi::c_void, u64) -> windows_core::HRESULT,
    pub WriteSingle: unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub WriteDouble: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub WriteDateTime:
        unsafe extern "system" fn(*mut core::ffi::c_void, windows_time::DateTime) -> windows_core::HRESULT,
    pub WriteTimeSpan:
        unsafe extern "system" fn(*mut core::ffi::c_void, windows_time::TimeSpan) -> windows_core::HRESULT,
    pub WriteString:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub MeasureString:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub StoreAsync:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub FlushAsync:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DetachBuffer:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DetachStream:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDataWriterFactory,
    IDataWriterFactory_Vtbl,
    0x338c67c2_8b84_4c2b_9c50_7b8767847a1f
);
impl windows_core::RuntimeType for IDataWriterFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDataWriterFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateDataWriter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IDependencyObject, IDependencyObject_Vtbl, 0x5c526665_f60e_4912_af59_5fe0680f089d);
impl windows_core::RuntimeType for IDependencyObject {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDependencyObject_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDependencyProperty,
    IDependencyProperty_Vtbl,
    0x85b13970_9bc4_4e96_acf1_30c8fd3d55c8
);
impl windows_core::RuntimeType for IDependencyProperty {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDependencyProperty_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDependencyPropertyChangedEventArgs,
    IDependencyPropertyChangedEventArgs_Vtbl,
    0x81212c2b_24d0_4957_abc3_224470a93a4e
);
impl windows_core::RuntimeType for IDependencyPropertyChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDependencyPropertyChangedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDeviceInformation,
    IDeviceInformation_Vtbl,
    0xaba0fb95_4398_489d_8e44_e6130927011f
);
impl windows_core::RuntimeType for IDeviceInformation {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDeviceInformation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IDispatcherTimer, IDispatcherTimer_Vtbl, 0xd160ce46_cd22_4f5f_8c97_40e61da3e2dc);
impl windows_core::RuntimeType for IDispatcherTimer {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IDispatcherTimer {
    pub fn Interval(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Interval)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetInterval(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetInterval)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn IsEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsEnabled)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn Tick<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let handler: EventHandler<windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<EventHandler<windows_core::IInspectable>, F>::new(
                &EventHandlerBox::<windows_core::IInspectable, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Tick)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(self.clone(), token__, windows_core::Interface::vtable(self).RemoveTick))
        }
    }
    pub fn Start(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).Start)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn Stop(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).Stop)(windows_core::Interface::as_raw(self)).ok() }
    }
}
#[repr(C)]
pub struct IDispatcherTimer_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Interval:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::TimeSpan) -> windows_core::HRESULT,
    pub SetInterval: unsafe extern "system" fn(*mut core::ffi::c_void, windows_time::TimeSpan) -> windows_core::HRESULT,
    pub IsEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub Tick:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveTick: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Start: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Stop: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDoubleTappedRoutedEventArgs,
    IDoubleTappedRoutedEventArgs_Vtbl,
    0xaf404424_26df_44f4_8714_9359249b62d3
);
impl windows_core::RuntimeType for IDoubleTappedRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDoubleTappedRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDownloadProgressEventArgs,
    IDownloadProgressEventArgs_Vtbl,
    0x7311e0d4_fe94_4e70_9b90_cdd47ac23afb
);
impl windows_core::RuntimeType for IDownloadProgressEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDownloadProgressEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IDragEventArgs, IDragEventArgs_Vtbl, 0xb440c7c3_02b4_4980_9342_25dae1c0f188);
impl windows_core::RuntimeType for IDragEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDragEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDragItemsStartingEventArgs,
    IDragItemsStartingEventArgs_Vtbl,
    0x71cf215c_daed_4783_aa11_dc574d2713e9
);
impl windows_core::RuntimeType for IDragItemsStartingEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDragItemsStartingEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IExceptionRoutedEventArgs,
    IExceptionRoutedEventArgs_Vtbl,
    0xdd9ff16a_4b62_4a6c_a49d_0671ef6136be
);
impl windows_core::RuntimeType for IExceptionRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IExceptionRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IFileActivatedEventArgs,
    IFileActivatedEventArgs_Vtbl,
    0xbb2afc33_93b1_42ed_8b26_236dd9c78496
);
impl windows_core::RuntimeType for IFileActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IFileActivatedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct IFileActivatedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IFileOpenPickerActivatedEventArgs,
    IFileOpenPickerActivatedEventArgs_Vtbl,
    0x72827082_5525_4bf2_bc09_1f5095d4964d
);
impl windows_core::RuntimeType for IFileOpenPickerActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(
    IFileOpenPickerActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
#[repr(C)]
pub struct IFileOpenPickerActivatedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IFileSavePickerActivatedEventArgs,
    IFileSavePickerActivatedEventArgs_Vtbl,
    0x81c19cf1_74e6_4387_82eb_bb8fd64b4346
);
impl windows_core::RuntimeType for IFileSavePickerActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(
    IFileSavePickerActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
#[repr(C)]
pub struct IFileSavePickerActivatedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IFlyoutBase, IFlyoutBase_Vtbl, 0x723eea0b_d12e_430d_a9f0_9bb32bbf9913);
impl windows_core::RuntimeType for IFlyoutBase {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IFlyoutBase {
    pub fn Placement(&self) -> windows_core::Result<FlyoutPlacementMode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Placement)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetPlacement(&self, value: FlyoutPlacementMode) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetPlacement)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn Opened<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let handler: EventHandler<windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<EventHandler<windows_core::IInspectable>, F>::new(
                &EventHandlerBox::<windows_core::IInspectable, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Opened)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveOpened,
            ))
        }
    }
    pub fn Closed<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let handler: EventHandler<windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<EventHandler<windows_core::IInspectable>, F>::new(
                &EventHandlerBox::<windows_core::IInspectable, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Closed)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveClosed,
            ))
        }
    }
    pub fn Opening<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let handler: EventHandler<windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<EventHandler<windows_core::IInspectable>, F>::new(
                &EventHandlerBox::<windows_core::IInspectable, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Opening)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveOpening,
            ))
        }
    }
    pub fn ShowAt<P0>(&self, placementtarget: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FrameworkElement>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ShowAt)(
                windows_core::Interface::as_raw(self),
                placementtarget.param().abi(),
            )
            .ok()
        }
    }
    pub fn Hide(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).Hide)(windows_core::Interface::as_raw(self)).ok() }
    }
}
#[repr(C)]
pub struct IFlyoutBase_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Placement: unsafe extern "system" fn(*mut core::ffi::c_void, *mut FlyoutPlacementMode) -> windows_core::HRESULT,
    pub SetPlacement: unsafe extern "system" fn(*mut core::ffi::c_void, FlyoutPlacementMode) -> windows_core::HRESULT,
    pub Opened:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveOpened: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Closed:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveClosed: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Opening:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveOpening: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ShowAt: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Hide: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IFontFamily, IFontFamily_Vtbl, 0x92467e64_d66a_4cf4_9322_3d23b3c0c361);
impl windows_core::RuntimeType for IFontFamily {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IFontFamily_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IFrameworkElement, IFrameworkElement_Vtbl, 0xa391d09b_4a99_4b7c_9d8d_6fa5d01f6fbf);
impl windows_core::RuntimeType for IFrameworkElement {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IFrameworkElement {
    pub fn Triggers(&self) -> windows_core::Result<TriggerCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Triggers)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Resources(&self) -> windows_core::Result<ResourceDictionary> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Resources)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetResources<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ResourceDictionary>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetResources)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Tag(&self) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Tag)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetTag<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetTag)(windows_core::Interface::as_raw(self), value.param().abi())
                .ok()
        }
    }
    pub fn Language(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Language)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn SetLanguage(&self, value: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetLanguage)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
            )
            .ok()
        }
    }
    pub fn ActualWidth(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ActualWidth)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn ActualHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ActualHeight)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn Width(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Width)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetWidth(&self, value: f64) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetWidth)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn Height(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Height)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetHeight(&self, value: f64) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetHeight)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn MinWidth(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MinWidth)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetMinWidth(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMinWidth)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn MaxWidth(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MaxWidth)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetMaxWidth(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMaxWidth)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn MinHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MinHeight)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetMinHeight(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMinHeight)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn MaxHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MaxHeight)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetMaxHeight(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMaxHeight)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn HorizontalAlignment(&self) -> windows_core::Result<HorizontalAlignment> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).HorizontalAlignment)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetHorizontalAlignment(&self, value: HorizontalAlignment) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetHorizontalAlignment)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn VerticalAlignment(&self) -> windows_core::Result<VerticalAlignment> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).VerticalAlignment)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetVerticalAlignment(&self, value: VerticalAlignment) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetVerticalAlignment)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn Margin(&self) -> windows_core::Result<Thickness> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Margin)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetMargin(&self, value: Thickness) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetMargin)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn Name(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Name)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn SetName(&self, value: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetName)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
            )
            .ok()
        }
    }
    pub fn BaseUri(&self) -> windows_core::Result<Uri> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BaseUri)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn DataContext(&self) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DataContext)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetDataContext<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetDataContext)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Style(&self) -> windows_core::Result<Style> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Style)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetStyle<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Style>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetStyle)(windows_core::Interface::as_raw(self), value.param().abi())
                .ok()
        }
    }
    pub fn Parent(&self) -> windows_core::Result<DependencyObject> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Parent)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn FlowDirection(&self) -> windows_core::Result<FlowDirection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FlowDirection)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetFlowDirection(&self, value: FlowDirection) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetFlowDirection)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn Loaded<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
    {
        let handler: RoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RoutedEventHandler, F>::new(
                &RoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Loaded)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveLoaded,
            ))
        }
    }
    pub fn Unloaded<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
    {
        let handler: RoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RoutedEventHandler, F>::new(
                &RoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Unloaded)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveUnloaded,
            ))
        }
    }
    pub fn SizeChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<SizeChangedEventArgs>) + 'static,
    {
        let handler: SizeChangedEventHandler = {
            let com = windows_core::imp::DelegateBox::<SizeChangedEventHandler, F>::new(
                &SizeChangedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).SizeChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveSizeChanged,
            ))
        }
    }
    pub fn LayoutUpdated<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let handler: EventHandler<windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<EventHandler<windows_core::IInspectable>, F>::new(
                &EventHandlerBox::<windows_core::IInspectable, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).LayoutUpdated)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveLayoutUpdated,
            ))
        }
    }
    pub fn FindName(&self, name: &str) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindName)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(name)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetBinding<P0, P1>(&self, dp: P0, binding: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<DependencyProperty>,
        P1: windows_core::Param<BindingBase>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetBinding)(
                windows_core::Interface::as_raw(self),
                dp.param().abi(),
                binding.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IFrameworkElement_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Triggers:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Resources:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetResources:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Tag: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetTag: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Language:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetLanguage: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ActualWidth: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub ActualHeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub Width: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetWidth: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub Height: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetHeight: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub MinWidth: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetMinWidth: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub MaxWidth: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetMaxWidth: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub MinHeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetMinHeight: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub MaxHeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetMaxHeight: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub HorizontalAlignment:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut HorizontalAlignment) -> windows_core::HRESULT,
    pub SetHorizontalAlignment:
        unsafe extern "system" fn(*mut core::ffi::c_void, HorizontalAlignment) -> windows_core::HRESULT,
    pub VerticalAlignment:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut VerticalAlignment) -> windows_core::HRESULT,
    pub SetVerticalAlignment:
        unsafe extern "system" fn(*mut core::ffi::c_void, VerticalAlignment) -> windows_core::HRESULT,
    pub Margin: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Thickness) -> windows_core::HRESULT,
    pub SetMargin: unsafe extern "system" fn(*mut core::ffi::c_void, Thickness) -> windows_core::HRESULT,
    pub Name: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetName: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub BaseUri:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DataContext:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetDataContext:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Style: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetStyle: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Parent: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub FlowDirection: unsafe extern "system" fn(*mut core::ffi::c_void, *mut FlowDirection) -> windows_core::HRESULT,
    pub SetFlowDirection: unsafe extern "system" fn(*mut core::ffi::c_void, FlowDirection) -> windows_core::HRESULT,
    pub Loaded:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveLoaded: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Unloaded:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveUnloaded: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub SizeChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveSizeChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub LayoutUpdated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveLayoutUpdated: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub FindName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetBinding: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IFrameworkTemplate,
    IFrameworkTemplate_Vtbl,
    0xa1e254d8_a446_4a27_9a9d_a0f59e1258a5
);
impl windows_core::RuntimeType for IFrameworkTemplate {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IFrameworkTemplate_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IGeneralTransform, IGeneralTransform_Vtbl, 0xa06798b7_a2ec_415f_ade2_eade9333f2c7);
impl windows_core::RuntimeType for IGeneralTransform {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IGeneralTransform_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IGeometry, IGeometry_Vtbl, 0xfa123889_0acd_417b_b62d_5ca1bf4dfc0e);
impl windows_core::RuntimeType for IGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IGeometry_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IGrid, IGrid_Vtbl, 0xfd104460_2e15_4ba3_8b8f_fa693a4161e9);
impl windows_core::RuntimeType for IGrid {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IGrid {
    pub fn RowDefinitions(&self) -> windows_core::Result<RowDefinitionCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RowDefinitions)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn ColumnDefinitions(&self) -> windows_core::Result<ColumnDefinitionCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ColumnDefinitions)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IGrid_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub RowDefinitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ColumnDefinitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IGridStatics, IGridStatics_Vtbl, 0x64fe2e9f_f951_42b6_a9ce_bb179af11595);
impl windows_core::RuntimeType for IGridStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IGridStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub RowProperty:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetRow:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetRow: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub ColumnProperty:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetColumn:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetColumn:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub RowSpanProperty:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetRowSpan:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetRowSpan:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub ColumnSpanProperty:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetColumnSpan:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetColumnSpan:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, i32) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IGridView, IGridView_Vtbl, 0x026ae934_b67e_4d80_8f72_8aa64b4d827b);
impl windows_core::RuntimeType for IGridView {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IGridView_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IGroupStyle, IGroupStyle_Vtbl, 0x71ca300b_b89e_4184_b0df_a9e95447de8d);
impl windows_core::RuntimeType for IGroupStyle {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IGroupStyle_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IGroupStyleSelector,
    IGroupStyleSelector_Vtbl,
    0x6dda119e_9a74_478f_a339_5bb52de6c51e
);
impl windows_core::RuntimeType for IGroupStyleSelector {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IGroupStyleSelector_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHoldingRoutedEventArgs,
    IHoldingRoutedEventArgs_Vtbl,
    0xc246ff23_d80d_44de_8db9_0d815e269ac0
);
impl windows_core::RuntimeType for IHoldingRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHoldingRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IHostName, IHostName_Vtbl, 0xbf8ecaad_ed96_49a7_9084_d416cae88dcb);
impl windows_core::RuntimeType for IHostName {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHostName_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpCacheDirectiveHeaderValueCollection,
    IHttpCacheDirectiveHeaderValueCollection_Vtbl,
    0x9a586b89_d5d0_4fbe_bd9d_b5b3636811b4
);
impl windows_core::RuntimeType for IHttpCacheDirectiveHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpCacheDirectiveHeaderValueCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IHttpClient, IHttpClient_Vtbl, 0x7fda1151_3574_4880_a8ba_e6b1e0061f3d);
impl windows_core::RuntimeType for IHttpClient {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IHttpClient {
    pub fn DeleteAsync<P0>(
        &self,
        uri: P0,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<HttpResponseMessage, HttpProgress>>
    where
        P0: windows_core::Param<Uri>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DeleteAsync)(
                windows_core::Interface::as_raw(self),
                uri.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetAsync<P0>(
        &self,
        uri: P0,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<HttpResponseMessage, HttpProgress>>
    where
        P0: windows_core::Param<Uri>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAsync)(
                windows_core::Interface::as_raw(self),
                uri.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetWithOptionAsync<P0>(
        &self,
        uri: P0,
        completionoption: HttpCompletionOption,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<HttpResponseMessage, HttpProgress>>
    where
        P0: windows_core::Param<Uri>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetWithOptionAsync)(
                windows_core::Interface::as_raw(self),
                uri.param().abi(),
                completionoption,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetBufferAsync<P0>(
        &self,
        uri: P0,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<IBuffer, HttpProgress>>
    where
        P0: windows_core::Param<Uri>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetBufferAsync)(
                windows_core::Interface::as_raw(self),
                uri.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetInputStreamAsync<P0>(
        &self,
        uri: P0,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<IInputStream, HttpProgress>>
    where
        P0: windows_core::Param<Uri>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetInputStreamAsync)(
                windows_core::Interface::as_raw(self),
                uri.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetStringAsync<P0>(
        &self,
        uri: P0,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<windows_core::HSTRING, HttpProgress>>
    where
        P0: windows_core::Param<Uri>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetStringAsync)(
                windows_core::Interface::as_raw(self),
                uri.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn PostAsync<P0, P1>(
        &self,
        uri: P0,
        content: P1,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<HttpResponseMessage, HttpProgress>>
    where
        P0: windows_core::Param<Uri>,
        P1: windows_core::Param<IHttpContent>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PostAsync)(
                windows_core::Interface::as_raw(self),
                uri.param().abi(),
                content.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn PutAsync<P0, P1>(
        &self,
        uri: P0,
        content: P1,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<HttpResponseMessage, HttpProgress>>
    where
        P0: windows_core::Param<Uri>,
        P1: windows_core::Param<IHttpContent>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PutAsync)(
                windows_core::Interface::as_raw(self),
                uri.param().abi(),
                content.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SendRequestAsync<P0>(
        &self,
        request: P0,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<HttpResponseMessage, HttpProgress>>
    where
        P0: windows_core::Param<HttpRequestMessage>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SendRequestAsync)(
                windows_core::Interface::as_raw(self),
                request.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SendRequestWithOptionAsync<P0>(
        &self,
        request: P0,
        completionoption: HttpCompletionOption,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<HttpResponseMessage, HttpProgress>>
    where
        P0: windows_core::Param<HttpRequestMessage>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SendRequestWithOptionAsync)(
                windows_core::Interface::as_raw(self),
                request.param().abi(),
                completionoption,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn DefaultRequestHeaders(&self) -> windows_core::Result<HttpRequestHeaderCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DefaultRequestHeaders)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IHttpClient_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub DeleteAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetWithOptionAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        HttpCompletionOption,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetBufferAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetInputStreamAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetStringAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub PostAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub PutAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SendRequestAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SendRequestWithOptionAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        HttpCompletionOption,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DefaultRequestHeaders:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IHttpConnectionOptionHeaderValueCollection,
    IHttpConnectionOptionHeaderValueCollection_Vtbl,
    0xe4f56c1d_5142_4e00_8e0f_019509337629
);
impl windows_core::RuntimeType for IHttpConnectionOptionHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpConnectionOptionHeaderValueCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IHttpContent, IHttpContent_Vtbl, 0x6b14a441_fba7_4bd2_af0a_839de7c295da);
impl windows_core::RuntimeType for IHttpContent {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IHttpContent, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct IHttpContent_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpContentCodingWithQualityHeaderValueCollection,
    IHttpContentCodingWithQualityHeaderValueCollection_Vtbl,
    0x7c0d753e_e899_4378_b5c8_412d820711cc
);
impl windows_core::RuntimeType for IHttpContentCodingWithQualityHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpContentCodingWithQualityHeaderValueCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpCookiePairHeaderValueCollection,
    IHttpCookiePairHeaderValueCollection_Vtbl,
    0xf3f44350_581e_4ecc_9f59_e507d04f06e6
);
impl windows_core::RuntimeType for IHttpCookiePairHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpCookiePairHeaderValueCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpCredentialsHeaderValue,
    IHttpCredentialsHeaderValue_Vtbl,
    0xc34cc3cb_542e_4177_a6c7_b674ce193fbf
);
impl windows_core::RuntimeType for IHttpCredentialsHeaderValue {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpCredentialsHeaderValue_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpExpectationHeaderValueCollection,
    IHttpExpectationHeaderValueCollection_Vtbl,
    0xe78521b3_a0e2_4ac4_9e66_79706cb9fd58
);
impl windows_core::RuntimeType for IHttpExpectationHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpExpectationHeaderValueCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpLanguageRangeWithQualityHeaderValueCollection,
    IHttpLanguageRangeWithQualityHeaderValueCollection_Vtbl,
    0x885d5abd_4b4f_480a_89ce_8aedcee6e3a0
);
impl windows_core::RuntimeType for IHttpLanguageRangeWithQualityHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpLanguageRangeWithQualityHeaderValueCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpMediaTypeWithQualityHeaderValueCollection,
    IHttpMediaTypeWithQualityHeaderValueCollection_Vtbl,
    0x3c0c6b73_1342_4587_a056_18d02ff67165
);
impl windows_core::RuntimeType for IHttpMediaTypeWithQualityHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpMediaTypeWithQualityHeaderValueCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpProductInfoHeaderValueCollection,
    IHttpProductInfoHeaderValueCollection_Vtbl,
    0x877df74a_d69b_44f8_ad4f_453af9c42ed0
);
impl windows_core::RuntimeType for IHttpProductInfoHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpProductInfoHeaderValueCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpRequestHeaderCollection,
    IHttpRequestHeaderCollection_Vtbl,
    0xaf40329b_b544_469b_86b9_ac3d466fea36
);
impl windows_core::RuntimeType for IHttpRequestHeaderCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IHttpRequestHeaderCollection {
    pub fn Accept(&self) -> windows_core::Result<HttpMediaTypeWithQualityHeaderValueCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Accept)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn AcceptEncoding(&self) -> windows_core::Result<HttpContentCodingWithQualityHeaderValueCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AcceptEncoding)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn AcceptLanguage(&self) -> windows_core::Result<HttpLanguageRangeWithQualityHeaderValueCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AcceptLanguage)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Authorization(&self) -> windows_core::Result<HttpCredentialsHeaderValue> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Authorization)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetAuthorization<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<HttpCredentialsHeaderValue>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetAuthorization)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn CacheControl(&self) -> windows_core::Result<HttpCacheDirectiveHeaderValueCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CacheControl)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Connection(&self) -> windows_core::Result<HttpConnectionOptionHeaderValueCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Connection)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Cookie(&self) -> windows_core::Result<HttpCookiePairHeaderValueCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Cookie)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Date(&self) -> windows_core::Result<windows_time::DateTime> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Date)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
                .and_then(|r__: windows_reference::IReference<windows_time::DateTime>| r__.Value())
        }
    }
    pub fn SetDate(&self, value: Option<windows_time::DateTime>) -> windows_core::Result<()> {
        let value__ = value.map(<windows_reference::IReference<windows_time::DateTime> as From<_>>::from);
        unsafe {
            (windows_core::Interface::vtable(self).SetDate)(
                windows_core::Interface::as_raw(self),
                windows_core::Param::param(value__.as_ref()).abi(),
            )
            .ok()
        }
    }
    pub fn Expect(&self) -> windows_core::Result<HttpExpectationHeaderValueCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Expect)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn From(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).From)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn SetFrom(&self, value: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetFrom)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
            )
            .ok()
        }
    }
    pub fn Host(&self) -> windows_core::Result<HostName> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Host)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetHost<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<HostName>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetHost)(windows_core::Interface::as_raw(self), value.param().abi())
                .ok()
        }
    }
    pub fn IfModifiedSince(&self) -> windows_core::Result<windows_time::DateTime> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IfModifiedSince)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
            .and_then(|r__: windows_reference::IReference<windows_time::DateTime>| r__.Value())
        }
    }
    pub fn SetIfModifiedSince(&self, value: Option<windows_time::DateTime>) -> windows_core::Result<()> {
        let value__ = value.map(<windows_reference::IReference<windows_time::DateTime> as From<_>>::from);
        unsafe {
            (windows_core::Interface::vtable(self).SetIfModifiedSince)(
                windows_core::Interface::as_raw(self),
                windows_core::Param::param(value__.as_ref()).abi(),
            )
            .ok()
        }
    }
    pub fn IfUnmodifiedSince(&self) -> windows_core::Result<windows_time::DateTime> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IfUnmodifiedSince)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
            .and_then(|r__: windows_reference::IReference<windows_time::DateTime>| r__.Value())
        }
    }
    pub fn SetIfUnmodifiedSince(&self, value: Option<windows_time::DateTime>) -> windows_core::Result<()> {
        let value__ = value.map(<windows_reference::IReference<windows_time::DateTime> as From<_>>::from);
        unsafe {
            (windows_core::Interface::vtable(self).SetIfUnmodifiedSince)(
                windows_core::Interface::as_raw(self),
                windows_core::Param::param(value__.as_ref()).abi(),
            )
            .ok()
        }
    }
    pub fn MaxForwards(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MaxForwards)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
                .and_then(|r__: windows_reference::IReference<u32>| r__.Value())
        }
    }
    pub fn SetMaxForwards(&self, value: Option<u32>) -> windows_core::Result<()> {
        let value__ = value.map(<windows_reference::IReference<u32> as From<_>>::from);
        unsafe {
            (windows_core::Interface::vtable(self).SetMaxForwards)(
                windows_core::Interface::as_raw(self),
                windows_core::Param::param(value__.as_ref()).abi(),
            )
            .ok()
        }
    }
    pub fn ProxyAuthorization(&self) -> windows_core::Result<HttpCredentialsHeaderValue> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ProxyAuthorization)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetProxyAuthorization<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<HttpCredentialsHeaderValue>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetProxyAuthorization)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Referer(&self) -> windows_core::Result<Uri> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Referer)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetReferer<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Uri>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetReferer)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn TransferEncoding(&self) -> windows_core::Result<HttpTransferCodingHeaderValueCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TransferEncoding)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn UserAgent(&self) -> windows_core::Result<HttpProductInfoHeaderValueCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).UserAgent)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Append(&self, name: &str, value: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).Append)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(name)),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
            )
            .ok()
        }
    }
    pub fn TryAppendWithoutValidation(&self, name: &str, value: &str) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryAppendWithoutValidation)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(name)),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IHttpRequestHeaderCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Accept: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub AcceptEncoding:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub AcceptLanguage:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Authorization:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetAuthorization:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CacheControl:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Connection:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Cookie: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Date: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetDate: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Expect: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub From: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetFrom: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Host: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetHost: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub IfModifiedSince:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetIfModifiedSince:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub IfUnmodifiedSince:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetIfUnmodifiedSince:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub MaxForwards:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetMaxForwards:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ProxyAuthorization:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetProxyAuthorization:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Referer:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetReferer: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub TransferEncoding:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub UserAgent:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Append: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub TryAppendWithoutValidation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut bool,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IHttpRequestMessage,
    IHttpRequestMessage_Vtbl,
    0xf5762b3c_74d4_4811_b5dc_9f8b4e2f9abf
);
impl windows_core::RuntimeType for IHttpRequestMessage {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpRequestMessage_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpResponseHeaderCollection,
    IHttpResponseHeaderCollection_Vtbl,
    0x7a990969_fa3f_41ed_aac6_bf957975c16b
);
impl windows_core::RuntimeType for IHttpResponseHeaderCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpResponseHeaderCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IHttpResponseMessage,
    IHttpResponseMessage_Vtbl,
    0xfee200fb_8664_44e0_95d9_42696199bffc
);
impl windows_core::RuntimeType for IHttpResponseMessage {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IHttpResponseMessage {
    pub fn Content(&self) -> windows_core::Result<IHttpContent> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Content)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetContent<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IHttpContent>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetContent)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Headers(&self) -> windows_core::Result<HttpResponseHeaderCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Headers)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn IsSuccessStatusCode(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsSuccessStatusCode)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ReasonPhrase(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ReasonPhrase)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                })
        }
    }
    pub fn SetReasonPhrase(&self, value: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetReasonPhrase)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
            )
            .ok()
        }
    }
    pub fn RequestMessage(&self) -> windows_core::Result<HttpRequestMessage> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RequestMessage)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetRequestMessage<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<HttpRequestMessage>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetRequestMessage)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Source(&self) -> windows_core::Result<HttpResponseMessageSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Source)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetSource(&self, value: HttpResponseMessageSource) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetSource)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn StatusCode(&self) -> windows_core::Result<HttpStatusCode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).StatusCode)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetStatusCode(&self, value: HttpStatusCode) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetStatusCode)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn Version(&self) -> windows_core::Result<HttpVersion> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Version)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetVersion(&self, value: HttpVersion) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetVersion)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn EnsureSuccessStatusCode(&self) -> windows_core::Result<HttpResponseMessage> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).EnsureSuccessStatusCode)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IHttpResponseMessage_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Content:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetContent: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Headers:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub IsSuccessStatusCode: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub ReasonPhrase:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetReasonPhrase:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub RequestMessage:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetRequestMessage:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Source:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut HttpResponseMessageSource) -> windows_core::HRESULT,
    pub SetSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, HttpResponseMessageSource) -> windows_core::HRESULT,
    pub StatusCode: unsafe extern "system" fn(*mut core::ffi::c_void, *mut HttpStatusCode) -> windows_core::HRESULT,
    pub SetStatusCode: unsafe extern "system" fn(*mut core::ffi::c_void, HttpStatusCode) -> windows_core::HRESULT,
    pub Version: unsafe extern "system" fn(*mut core::ffi::c_void, *mut HttpVersion) -> windows_core::HRESULT,
    pub SetVersion: unsafe extern "system" fn(*mut core::ffi::c_void, HttpVersion) -> windows_core::HRESULT,
    pub EnsureSuccessStatusCode:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IHttpTransferCodingHeaderValueCollection,
    IHttpTransferCodingHeaderValueCollection_Vtbl,
    0x202c8c34_2c03_49b8_9665_73e27cb2fc79
);
impl windows_core::RuntimeType for IHttpTransferCodingHeaderValueCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IHttpTransferCodingHeaderValueCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IIdleDispatchedHandlerArgs,
    IIdleDispatchedHandlerArgs_Vtbl,
    0x98bb6a24_dc1c_43cb_b4ed_d1c0eb2391f3
);
impl windows_core::RuntimeType for IIdleDispatchedHandlerArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IIdleDispatchedHandlerArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IImage, IImage_Vtbl, 0x495b7402_9af3_4e50_aa90_03388f3086d2);
impl windows_core::RuntimeType for IImage {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IImage {
    pub fn Source(&self) -> windows_core::Result<ImageSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Source)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetSource<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImageSource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetSource)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Stretch(&self) -> windows_core::Result<Stretch> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Stretch)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetStretch(&self, value: Stretch) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetStretch)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn NineGrid(&self) -> windows_core::Result<Thickness> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).NineGrid)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetNineGrid(&self, value: Thickness) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetNineGrid)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn PlayToSource(&self) -> windows_core::Result<PlayToSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PlayToSource)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn ImageFailed<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ExceptionRoutedEventArgs>) + 'static,
    {
        let handler: ExceptionRoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<ExceptionRoutedEventHandler, F>::new(
                &ExceptionRoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ImageFailed)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveImageFailed,
            ))
        }
    }
    pub fn ImageOpened<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
    {
        let handler: RoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RoutedEventHandler, F>::new(
                &RoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ImageOpened)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveImageOpened,
            ))
        }
    }
}
#[repr(C)]
pub struct IImage_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Source: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetSource: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Stretch: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Stretch) -> windows_core::HRESULT,
    pub SetStretch: unsafe extern "system" fn(*mut core::ffi::c_void, Stretch) -> windows_core::HRESULT,
    pub NineGrid: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Thickness) -> windows_core::HRESULT,
    pub SetNineGrid: unsafe extern "system" fn(*mut core::ffi::c_void, Thickness) -> windows_core::HRESULT,
    pub PlayToSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ImageFailed:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveImageFailed: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ImageOpened:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveImageOpened: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IImageSource, IImageSource_Vtbl, 0x737ef309_ea41_4d96_a71c_98e98efcab07);
impl windows_core::RuntimeType for IImageSource {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IImageSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IInline, IInline_Vtbl, 0x0c92712d_1bc9_4931_8cb1_1aeadf1cc685);
impl windows_core::RuntimeType for IInline {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IInline_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IInputScope, IInputScope_Vtbl, 0x5c0f85f3_f9d8_4220_b666_045d074d9bfa);
impl windows_core::RuntimeType for IInputScope {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IInputScope_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IInputStream, IInputStream_Vtbl, 0x905a0fe2_bc53_11df_8c49_001e4fc686da);
impl windows_core::RuntimeType for IInputStream {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IInputStream, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(IInputStream, IClosable);
impl IInputStream {
    pub fn ReadAsync<P0>(
        &self,
        buffer: P0,
        count: u32,
        options: InputStreamOptions,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<IBuffer, u32>>
    where
        P0: windows_core::Param<IBuffer>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ReadAsync)(
                windows_core::Interface::as_raw(self),
                buffer.param().abi(),
                count,
                options,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IInputStream_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ReadAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        InputStreamOptions,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IItemClickEventArgs,
    IItemClickEventArgs_Vtbl,
    0x9c314242_f29a_494d_a3a5_d4c7b2a46852
);
impl windows_core::RuntimeType for IItemClickEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IItemClickEventArgs {
    pub fn ClickedItem(&self) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ClickedItem)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IItemClickEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ClickedItem:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IItemContainerGenerator,
    IItemContainerGenerator_Vtbl,
    0x40765f00_83f0_4d7f_b1b8_f19de4f1d5da
);
impl windows_core::RuntimeType for IItemContainerGenerator {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IItemContainerGenerator_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IItemsControl, IItemsControl_Vtbl, 0xf4a91dd8_d979_4381_8652_bda0342a765e);
impl windows_core::RuntimeType for IItemsControl {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IItemsControl {
    pub fn ItemsSource(&self) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemsSource)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetItemsSource<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetItemsSource)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Items(&self) -> windows_core::Result<ItemCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Items)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn ItemTemplate(&self) -> windows_core::Result<DataTemplate> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemTemplate)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetItemTemplate<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<DataTemplate>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetItemTemplate)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn ItemTemplateSelector(&self) -> windows_core::Result<DataTemplateSelector> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemTemplateSelector)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetItemTemplateSelector<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<DataTemplateSelector>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetItemTemplateSelector)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn ItemsPanel(&self) -> windows_core::Result<ItemsPanelTemplate> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemsPanel)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetItemsPanel<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ItemsPanelTemplate>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetItemsPanel)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn DisplayMemberPath(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DisplayMemberPath)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| {
                let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                hstring.to_string_lossy()
            })
        }
    }
    pub fn SetDisplayMemberPath(&self, value: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetDisplayMemberPath)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
            )
            .ok()
        }
    }
    pub fn ItemContainerStyle(&self) -> windows_core::Result<Style> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemContainerStyle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetItemContainerStyle<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Style>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetItemContainerStyle)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn ItemContainerStyleSelector(&self) -> windows_core::Result<StyleSelector> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemContainerStyleSelector)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetItemContainerStyleSelector<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<StyleSelector>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetItemContainerStyleSelector)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn ItemContainerGenerator(&self) -> windows_core::Result<ItemContainerGenerator> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemContainerGenerator)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn ItemContainerTransitions(&self) -> windows_core::Result<TransitionCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemContainerTransitions)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetItemContainerTransitions<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<TransitionCollection>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetItemContainerTransitions)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn GroupStyle(&self) -> windows_core::Result<windows_collections::IObservableVector<GroupStyle>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GroupStyle)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GroupStyleSelector(&self) -> windows_core::Result<GroupStyleSelector> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GroupStyleSelector)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetGroupStyleSelector<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<GroupStyleSelector>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetGroupStyleSelector)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn IsGrouping(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsGrouping)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IItemsControl_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ItemsSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetItemsSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Items: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ItemTemplate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetItemTemplate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ItemTemplateSelector:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetItemTemplateSelector:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ItemsPanel:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetItemsPanel:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DisplayMemberPath:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetDisplayMemberPath:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ItemContainerStyle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetItemContainerStyle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ItemContainerStyleSelector:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetItemContainerStyleSelector:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ItemContainerGenerator:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ItemContainerTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetItemContainerTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GroupStyle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GroupStyleSelector:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetGroupStyleSelector:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub IsGrouping: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IItemsControl2, IItemsControl2_Vtbl, 0x754c3266_0529_45e3_8748_bf747d158357);
impl windows_core::RuntimeType for IItemsControl2 {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IItemsControl2 {
    pub fn ItemsPanelRoot(&self) -> windows_core::Result<Panel> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemsPanelRoot)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IItemsControl2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ItemsPanelRoot:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IItemsPanelTemplate,
    IItemsPanelTemplate_Vtbl,
    0x14ead768_34b2_4b97_bf3c_e8d73230a2aa
);
impl windows_core::RuntimeType for IItemsPanelTemplate {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IItemsPanelTemplate_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IItemsWrapGrid, IItemsWrapGrid_Vtbl, 0xddfe6b5f_ba88_4a7d_8a5b_e58caf0f4e2d);
impl windows_core::RuntimeType for IItemsWrapGrid {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IItemsWrapGrid {
    pub fn GroupPadding(&self) -> windows_core::Result<Thickness> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GroupPadding)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetGroupPadding(&self, value: Thickness) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetGroupPadding)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn Orientation(&self) -> windows_core::Result<Orientation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Orientation)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetOrientation(&self, value: Orientation) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetOrientation)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn MaximumRowsOrColumns(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MaximumRowsOrColumns)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetMaximumRowsOrColumns(&self, value: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMaximumRowsOrColumns)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn ItemWidth(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemWidth)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetItemWidth(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetItemWidth)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn ItemHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ItemHeight)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetItemHeight(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetItemHeight)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn FirstCacheIndex(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FirstCacheIndex)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn FirstVisibleIndex(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FirstVisibleIndex)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn LastVisibleIndex(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LastVisibleIndex)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn LastCacheIndex(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LastCacheIndex)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn ScrollingDirection(&self) -> windows_core::Result<PanelScrollingDirection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ScrollingDirection)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn GroupHeaderPlacement(&self) -> windows_core::Result<GroupHeaderPlacement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GroupHeaderPlacement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetGroupHeaderPlacement(&self, value: GroupHeaderPlacement) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetGroupHeaderPlacement)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn CacheLength(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CacheLength)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetCacheLength(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetCacheLength)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
}
#[repr(C)]
pub struct IItemsWrapGrid_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub GroupPadding: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Thickness) -> windows_core::HRESULT,
    pub SetGroupPadding: unsafe extern "system" fn(*mut core::ffi::c_void, Thickness) -> windows_core::HRESULT,
    pub Orientation: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Orientation) -> windows_core::HRESULT,
    pub SetOrientation: unsafe extern "system" fn(*mut core::ffi::c_void, Orientation) -> windows_core::HRESULT,
    pub MaximumRowsOrColumns: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetMaximumRowsOrColumns: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub ItemWidth: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetItemWidth: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub ItemHeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetItemHeight: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub FirstCacheIndex: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub FirstVisibleIndex: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub LastVisibleIndex: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub LastCacheIndex: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub ScrollingDirection:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut PanelScrollingDirection) -> windows_core::HRESULT,
    pub GroupHeaderPlacement:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GroupHeaderPlacement) -> windows_core::HRESULT,
    pub SetGroupHeaderPlacement:
        unsafe extern "system" fn(*mut core::ffi::c_void, GroupHeaderPlacement) -> windows_core::HRESULT,
    pub CacheLength: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetCacheLength: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IItemsWrapGrid2, IItemsWrapGrid2_Vtbl, 0x80c21c0f_63ec_4258_bd61_d4a6956c864a);
impl windows_core::RuntimeType for IItemsWrapGrid2 {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IItemsWrapGrid2 {
    pub fn AreStickyGroupHeadersEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AreStickyGroupHeadersEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetAreStickyGroupHeadersEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetAreStickyGroupHeadersEnabled)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IItemsWrapGrid2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub AreStickyGroupHeadersEnabled:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetAreStickyGroupHeadersEnabled:
        unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IKeyRoutedEventArgs,
    IKeyRoutedEventArgs_Vtbl,
    0xd4cd3dfe_4079_42e9_a39a_3095d3f049c6
);
impl windows_core::RuntimeType for IKeyRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IKeyRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ILaunchActivatedEventArgs,
    ILaunchActivatedEventArgs_Vtbl,
    0xfbc93e26_a14a_4b4f_82b0_33bed920af52
);
impl windows_core::RuntimeType for ILaunchActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(ILaunchActivatedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct ILaunchActivatedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IListView, IListView_Vtbl, 0xf6ce8c6d_fe96_41ad_a64a_c2b81c4af7f8);
impl windows_core::RuntimeType for IListView {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IListView_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IListViewBase, IListViewBase_Vtbl, 0x3d0813ba_6890_4537_bfe5_796d9458edd6);
impl windows_core::RuntimeType for IListViewBase {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IListViewBase {
    pub fn SelectedItems(&self) -> windows_core::Result<IVector<windows_core::IInspectable>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SelectedItems)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SelectionMode(&self) -> windows_core::Result<ListViewSelectionMode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SelectionMode)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetSelectionMode(&self, value: ListViewSelectionMode) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetSelectionMode)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn IsSwipeEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsSwipeEnabled)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetIsSwipeEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsSwipeEnabled)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn CanDragItems(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CanDragItems)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetCanDragItems(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetCanDragItems)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn CanReorderItems(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CanReorderItems)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetCanReorderItems(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetCanReorderItems)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn IsItemClickEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsItemClickEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIsItemClickEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsItemClickEnabled)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn DataFetchSize(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DataFetchSize)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetDataFetchSize(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetDataFetchSize)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn IncrementalLoadingThreshold(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IncrementalLoadingThreshold)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIncrementalLoadingThreshold(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIncrementalLoadingThreshold)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn IncrementalLoadingTrigger(&self) -> windows_core::Result<IncrementalLoadingTrigger> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IncrementalLoadingTrigger)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIncrementalLoadingTrigger(&self, value: IncrementalLoadingTrigger) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIncrementalLoadingTrigger)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn ItemClick<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ItemClickEventArgs>) + 'static,
    {
        let handler: ItemClickEventHandler = {
            let com = windows_core::imp::DelegateBox::<ItemClickEventHandler, F>::new(
                &ItemClickEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ItemClick)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveItemClick,
            ))
        }
    }
    pub fn DragItemsStarting<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DragItemsStartingEventArgs>) + 'static,
    {
        let handler: DragItemsStartingEventHandler = {
            let com = windows_core::imp::DelegateBox::<DragItemsStartingEventHandler, F>::new(
                &DragItemsStartingEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DragItemsStarting)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDragItemsStarting,
            ))
        }
    }
    pub fn ScrollIntoView<P0>(&self, item: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ScrollIntoView)(
                windows_core::Interface::as_raw(self),
                item.param().abi(),
            )
            .ok()
        }
    }
    pub fn SelectAll(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SelectAll)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn LoadMoreItemsAsync(&self) -> windows_core::Result<windows_future::IAsyncOperation<LoadMoreItemsResult>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LoadMoreItemsAsync)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn ScrollIntoViewWithAlignment<P0>(
        &self,
        item: P0,
        alignment: ScrollIntoViewAlignment,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ScrollIntoViewWithAlignment)(
                windows_core::Interface::as_raw(self),
                item.param().abi(),
                alignment,
            )
            .ok()
        }
    }
    pub fn Header(&self) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Header)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetHeader<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetHeader)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn HeaderTemplate(&self) -> windows_core::Result<DataTemplate> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).HeaderTemplate)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetHeaderTemplate<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<DataTemplate>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetHeaderTemplate)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn HeaderTransitions(&self) -> windows_core::Result<TransitionCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).HeaderTransitions)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetHeaderTransitions<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<TransitionCollection>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetHeaderTransitions)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IListViewBase_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub SelectedItems:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SelectionMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut ListViewSelectionMode) -> windows_core::HRESULT,
    pub SetSelectionMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, ListViewSelectionMode) -> windows_core::HRESULT,
    pub IsSwipeEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsSwipeEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub CanDragItems: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetCanDragItems: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub CanReorderItems: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetCanReorderItems: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub IsItemClickEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsItemClickEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub DataFetchSize: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetDataFetchSize: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub IncrementalLoadingThreshold:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetIncrementalLoadingThreshold: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub IncrementalLoadingTrigger:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut IncrementalLoadingTrigger) -> windows_core::HRESULT,
    pub SetIncrementalLoadingTrigger:
        unsafe extern "system" fn(*mut core::ffi::c_void, IncrementalLoadingTrigger) -> windows_core::HRESULT,
    pub ItemClick:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveItemClick: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub DragItemsStarting:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDragItemsStarting: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ScrollIntoView:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SelectAll: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub LoadMoreItemsAsync:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ScrollIntoViewWithAlignment: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        ScrollIntoViewAlignment,
    ) -> windows_core::HRESULT,
    pub Header: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetHeader: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub HeaderTemplate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetHeaderTemplate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub HeaderTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetHeaderTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IListViewBase2, IListViewBase2_Vtbl, 0xd1c2a4f7_78a3_4491_86e0_2ddebc007ac5);
impl windows_core::RuntimeType for IListViewBase2 {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IListViewBase2 {
    pub fn ShowsScrollingPlaceholders(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ShowsScrollingPlaceholders)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetShowsScrollingPlaceholders(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetShowsScrollingPlaceholders)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn ContainerContentChanging<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<ListViewBase>, windows_core::Ref<ContainerContentChangingEventArgs>) + 'static,
    {
        let handler: TypedEventHandler<ListViewBase, ContainerContentChangingEventArgs> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<ListViewBase, ContainerContentChangingEventArgs>,
                F,
            >::new(
                &TypedEventHandlerBox::<ListViewBase, ContainerContentChangingEventArgs, F>::VTABLE, handler
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ContainerContentChanging)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveContainerContentChanging,
            ))
        }
    }
    pub fn SetDesiredContainerUpdateDuration(&self, duration: windows_time::TimeSpan) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetDesiredContainerUpdateDuration)(
                windows_core::Interface::as_raw(self),
                duration,
            )
            .ok()
        }
    }
    pub fn Footer(&self) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Footer)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetFooter<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetFooter)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn FooterTemplate(&self) -> windows_core::Result<DataTemplate> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FooterTemplate)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetFooterTemplate<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<DataTemplate>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetFooterTemplate)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn FooterTransitions(&self) -> windows_core::Result<TransitionCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FooterTransitions)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetFooterTransitions<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<TransitionCollection>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetFooterTransitions)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IListViewBase2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ShowsScrollingPlaceholders:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetShowsScrollingPlaceholders: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub ContainerContentChanging:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveContainerContentChanging: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub SetDesiredContainerUpdateDuration:
        unsafe extern "system" fn(*mut core::ffi::c_void, windows_time::TimeSpan) -> windows_core::HRESULT,
    pub Footer: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetFooter: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub FooterTemplate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetFooterTemplate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub FooterTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetFooterTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IListViewItem, IListViewItem_Vtbl, 0xb7bb4305_3dd9_43de_a8c0_c472f085bc11);
impl windows_core::RuntimeType for IListViewItem {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IListViewItem_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IManipulationCompletedRoutedEventArgs,
    IManipulationCompletedRoutedEventArgs_Vtbl,
    0xb5ad9b23_2f41_498e_8319_015ee8a75346
);
impl windows_core::RuntimeType for IManipulationCompletedRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IManipulationCompletedRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IManipulationDeltaRoutedEventArgs,
    IManipulationDeltaRoutedEventArgs_Vtbl,
    0x400d5794_4c6f_491d_82d6_3517109399c6
);
impl windows_core::RuntimeType for IManipulationDeltaRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IManipulationDeltaRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IManipulationInertiaStartingRoutedEventArgs,
    IManipulationInertiaStartingRoutedEventArgs_Vtbl,
    0x246a91a9_ca43_4c0b_acef_81e8b8147520
);
impl windows_core::RuntimeType for IManipulationInertiaStartingRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IManipulationInertiaStartingRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IManipulationStartedRoutedEventArgs,
    IManipulationStartedRoutedEventArgs_Vtbl,
    0x5db1aa05_9f80_48b6_ae6c_4f119de8ff13
);
impl windows_core::RuntimeType for IManipulationStartedRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IManipulationStartedRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IManipulationStartingRoutedEventArgs,
    IManipulationStartingRoutedEventArgs_Vtbl,
    0x18d636b7_53a4_4c15_a498_f3a9ca212a42
);
impl windows_core::RuntimeType for IManipulationStartingRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IManipulationStartingRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IMediaBinder, IMediaBinder_Vtbl, 0x2b7e40aa_de07_424f_83f1_f1de46c4fa2e);
impl windows_core::RuntimeType for IMediaBinder {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaBinder_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IMediaBreakManager,
    IMediaBreakManager_Vtbl,
    0xa854ddb1_feb4_4d9b_9d97_0fdbe58e5e39
);
impl windows_core::RuntimeType for IMediaBreakManager {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaBreakManager_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IMediaPlaybackCommandManager,
    IMediaPlaybackCommandManager_Vtbl,
    0x5acee5a6_5cb6_4a5a_8521_cc86b1c1ed37
);
impl windows_core::RuntimeType for IMediaPlaybackCommandManager {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaPlaybackCommandManager_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IMediaPlaybackItem,
    IMediaPlaybackItem_Vtbl,
    0x047097d2_e4af_48ab_b283_6929e674ece2
);
impl windows_core::RuntimeType for IMediaPlaybackItem {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IMediaPlaybackItem {
    pub fn AudioTracksChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackItem>, windows_core::Ref<windows_collections::IVectorChangedEventArgs>)
            + 'static,
    {
        let handler: TypedEventHandler<MediaPlaybackItem, windows_collections::IVectorChangedEventArgs> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackItem, windows_collections::IVectorChangedEventArgs>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackItem, windows_collections::IVectorChangedEventArgs, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).AudioTracksChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveAudioTracksChanged,
            ))
        }
    }
    pub fn VideoTracksChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackItem>, windows_core::Ref<windows_collections::IVectorChangedEventArgs>)
            + 'static,
    {
        let handler: TypedEventHandler<MediaPlaybackItem, windows_collections::IVectorChangedEventArgs> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackItem, windows_collections::IVectorChangedEventArgs>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackItem, windows_collections::IVectorChangedEventArgs, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).VideoTracksChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveVideoTracksChanged,
            ))
        }
    }
    pub fn TimedMetadataTracksChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackItem>, windows_core::Ref<windows_collections::IVectorChangedEventArgs>)
            + 'static,
    {
        let handler: TypedEventHandler<MediaPlaybackItem, windows_collections::IVectorChangedEventArgs> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackItem, windows_collections::IVectorChangedEventArgs>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackItem, windows_collections::IVectorChangedEventArgs, F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).TimedMetadataTracksChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveTimedMetadataTracksChanged,
            ))
        }
    }
    pub fn Source(&self) -> windows_core::Result<MediaSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Source)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn AudioTracks(&self) -> windows_core::Result<MediaPlaybackAudioTrackList> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AudioTracks)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn VideoTracks(&self) -> windows_core::Result<MediaPlaybackVideoTrackList> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).VideoTracks)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn TimedMetadataTracks(&self) -> windows_core::Result<MediaPlaybackTimedMetadataTrackList> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TimedMetadataTracks)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IMediaPlaybackItem_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub AudioTracksChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveAudioTracksChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub VideoTracksChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveVideoTracksChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub TimedMetadataTracksChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveTimedMetadataTracksChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Source: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub AudioTracks:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub VideoTracks:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub TimedMetadataTracks:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IMediaPlaybackItemFactory,
    IMediaPlaybackItemFactory_Vtbl,
    0x7133fce1_1769_4ff9_a7c1_38d2c4d42360
);
impl windows_core::RuntimeType for IMediaPlaybackItemFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaPlaybackItemFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Create: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IMediaPlaybackSession,
    IMediaPlaybackSession_Vtbl,
    0xc32b683d_0407_41ba_8946_8b345a5a5435
);
impl windows_core::RuntimeType for IMediaPlaybackSession {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IMediaPlaybackSession {
    pub fn PlaybackStateChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackSession>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackSession, windows_core::IInspectable, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PlaybackStateChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePlaybackStateChanged,
            ))
        }
    }
    pub fn PlaybackRateChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackSession>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackSession, windows_core::IInspectable, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PlaybackRateChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePlaybackRateChanged,
            ))
        }
    }
    pub fn SeekCompleted<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackSession>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackSession, windows_core::IInspectable, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).SeekCompleted)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveSeekCompleted,
            ))
        }
    }
    pub fn BufferingStarted<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackSession>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackSession, windows_core::IInspectable, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).BufferingStarted)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveBufferingStarted,
            ))
        }
    }
    pub fn BufferingEnded<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackSession>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackSession, windows_core::IInspectable, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).BufferingEnded)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveBufferingEnded,
            ))
        }
    }
    pub fn BufferingProgressChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackSession>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackSession, windows_core::IInspectable, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).BufferingProgressChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveBufferingProgressChanged,
            ))
        }
    }
    pub fn DownloadProgressChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackSession>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackSession, windows_core::IInspectable, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DownloadProgressChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDownloadProgressChanged,
            ))
        }
    }
    pub fn NaturalDurationChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackSession>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackSession, windows_core::IInspectable, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).NaturalDurationChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveNaturalDurationChanged,
            ))
        }
    }
    pub fn PositionChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackSession>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackSession, windows_core::IInspectable, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PositionChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePositionChanged,
            ))
        }
    }
    pub fn NaturalVideoSizeChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlaybackSession>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlaybackSession, windows_core::IInspectable>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlaybackSession, windows_core::IInspectable, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).NaturalVideoSizeChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveNaturalVideoSizeChanged,
            ))
        }
    }
    pub fn MediaPlayer(&self) -> windows_core::Result<MediaPlayer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MediaPlayer)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn NaturalDuration(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).NaturalDuration)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn Position(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Position)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetPosition(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetPosition)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn PlaybackState(&self) -> windows_core::Result<MediaPlaybackState> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PlaybackState)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn CanSeek(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CanSeek)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn CanPause(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CanPause)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn IsProtected(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsProtected)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn PlaybackRate(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PlaybackRate)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetPlaybackRate(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetPlaybackRate)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn BufferingProgress(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BufferingProgress)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn DownloadProgress(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DownloadProgress)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn NaturalVideoHeight(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).NaturalVideoHeight)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn NaturalVideoWidth(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).NaturalVideoWidth)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn NormalizedSourceRect(&self) -> windows_core::Result<Rect> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).NormalizedSourceRect)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetNormalizedSourceRect(&self, value: Rect) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetNormalizedSourceRect)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn StereoscopicVideoPackingMode(&self) -> windows_core::Result<StereoscopicVideoPackingMode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).StereoscopicVideoPackingMode)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetStereoscopicVideoPackingMode(&self, value: StereoscopicVideoPackingMode) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetStereoscopicVideoPackingMode)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IMediaPlaybackSession_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub PlaybackStateChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePlaybackStateChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PlaybackRateChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePlaybackRateChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub SeekCompleted:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveSeekCompleted: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub BufferingStarted:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveBufferingStarted: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub BufferingEnded:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveBufferingEnded: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub BufferingProgressChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveBufferingProgressChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub DownloadProgressChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDownloadProgressChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub NaturalDurationChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveNaturalDurationChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PositionChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePositionChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub NaturalVideoSizeChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveNaturalVideoSizeChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub MediaPlayer:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub NaturalDuration:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::TimeSpan) -> windows_core::HRESULT,
    pub Position:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::TimeSpan) -> windows_core::HRESULT,
    pub SetPosition: unsafe extern "system" fn(*mut core::ffi::c_void, windows_time::TimeSpan) -> windows_core::HRESULT,
    pub PlaybackState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut MediaPlaybackState) -> windows_core::HRESULT,
    pub CanSeek: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub CanPause: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub IsProtected: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub PlaybackRate: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetPlaybackRate: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub BufferingProgress: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub DownloadProgress: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub NaturalVideoHeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub NaturalVideoWidth: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub NormalizedSourceRect: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Rect) -> windows_core::HRESULT,
    pub SetNormalizedSourceRect: unsafe extern "system" fn(*mut core::ffi::c_void, Rect) -> windows_core::HRESULT,
    pub StereoscopicVideoPackingMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut StereoscopicVideoPackingMode) -> windows_core::HRESULT,
    pub SetStereoscopicVideoPackingMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, StereoscopicVideoPackingMode) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IMediaPlaybackSource,
    IMediaPlaybackSource_Vtbl,
    0xef9dc2bc_9317_4696_b051_2bad643177b5
);
impl windows_core::RuntimeType for IMediaPlaybackSource {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IMediaPlaybackSource, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct IMediaPlaybackSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IMediaPlayer, IMediaPlayer_Vtbl, 0x381a83cb_6fff_499b_8d64_2885dfc1249e);
impl windows_core::RuntimeType for IMediaPlayer {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IMediaPlayer {
    pub fn AutoPlay(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AutoPlay)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetAutoPlay(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetAutoPlay)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn NaturalDuration(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).NaturalDuration)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn Position(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Position)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetPosition(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetPosition)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn BufferingProgress(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BufferingProgress)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn CurrentState(&self) -> windows_core::Result<MediaPlayerState> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentState)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn CanSeek(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CanSeek)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn CanPause(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CanPause)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn IsLoopingEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsLoopingEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIsLoopingEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsLoopingEnabled)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn IsProtected(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsProtected)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn IsMuted(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsMuted)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetIsMuted(&self, value: bool) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetIsMuted)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn PlaybackRate(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PlaybackRate)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetPlaybackRate(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetPlaybackRate)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn Volume(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Volume)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetVolume(&self, value: f64) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetVolume)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn PlaybackMediaMarkers(&self) -> windows_core::Result<PlaybackMediaMarkerSequence> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PlaybackMediaMarkers)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn MediaOpened<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, windows_core::IInspectable> = {
            let com =
                windows_core::imp::DelegateBox::<TypedEventHandler<MediaPlayer, windows_core::IInspectable>, F>::new(
                    &TypedEventHandlerBox::<MediaPlayer, windows_core::IInspectable, F>::VTABLE,
                    value,
                );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).MediaOpened)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveMediaOpened,
            ))
        }
    }
    pub fn MediaEnded<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, windows_core::IInspectable> = {
            let com =
                windows_core::imp::DelegateBox::<TypedEventHandler<MediaPlayer, windows_core::IInspectable>, F>::new(
                    &TypedEventHandlerBox::<MediaPlayer, windows_core::IInspectable, F>::VTABLE,
                    value,
                );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).MediaEnded)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveMediaEnded,
            ))
        }
    }
    pub fn MediaFailed<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<MediaPlayerFailedEventArgs>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, MediaPlayerFailedEventArgs> = {
            let com =
                windows_core::imp::DelegateBox::<TypedEventHandler<MediaPlayer, MediaPlayerFailedEventArgs>, F>::new(
                    &TypedEventHandlerBox::<MediaPlayer, MediaPlayerFailedEventArgs, F>::VTABLE,
                    value,
                );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).MediaFailed)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveMediaFailed,
            ))
        }
    }
    pub fn CurrentStateChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, windows_core::IInspectable> = {
            let com =
                windows_core::imp::DelegateBox::<TypedEventHandler<MediaPlayer, windows_core::IInspectable>, F>::new(
                    &TypedEventHandlerBox::<MediaPlayer, windows_core::IInspectable, F>::VTABLE,
                    value,
                );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).CurrentStateChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveCurrentStateChanged,
            ))
        }
    }
    pub fn PlaybackMediaMarkerReached<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<PlaybackMediaMarkerReachedEventArgs>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, PlaybackMediaMarkerReachedEventArgs> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlayer, PlaybackMediaMarkerReachedEventArgs>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlayer, PlaybackMediaMarkerReachedEventArgs, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PlaybackMediaMarkerReached)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePlaybackMediaMarkerReached,
            ))
        }
    }
    pub fn MediaPlayerRateChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<MediaPlayerRateChangedEventArgs>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, MediaPlayerRateChangedEventArgs> = {
            let com = windows_core::imp::DelegateBox::<
                TypedEventHandler<MediaPlayer, MediaPlayerRateChangedEventArgs>,
                F,
            >::new(
                &TypedEventHandlerBox::<MediaPlayer, MediaPlayerRateChangedEventArgs, F>::VTABLE, value
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).MediaPlayerRateChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveMediaPlayerRateChanged,
            ))
        }
    }
    pub fn VolumeChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, windows_core::IInspectable> = {
            let com =
                windows_core::imp::DelegateBox::<TypedEventHandler<MediaPlayer, windows_core::IInspectable>, F>::new(
                    &TypedEventHandlerBox::<MediaPlayer, windows_core::IInspectable, F>::VTABLE,
                    value,
                );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).VolumeChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveVolumeChanged,
            ))
        }
    }
    pub fn SeekCompleted<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, windows_core::IInspectable> = {
            let com =
                windows_core::imp::DelegateBox::<TypedEventHandler<MediaPlayer, windows_core::IInspectable>, F>::new(
                    &TypedEventHandlerBox::<MediaPlayer, windows_core::IInspectable, F>::VTABLE,
                    value,
                );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).SeekCompleted)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveSeekCompleted,
            ))
        }
    }
    pub fn BufferingStarted<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, windows_core::IInspectable> = {
            let com =
                windows_core::imp::DelegateBox::<TypedEventHandler<MediaPlayer, windows_core::IInspectable>, F>::new(
                    &TypedEventHandlerBox::<MediaPlayer, windows_core::IInspectable, F>::VTABLE,
                    value,
                );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).BufferingStarted)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveBufferingStarted,
            ))
        }
    }
    pub fn BufferingEnded<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, windows_core::IInspectable> = {
            let com =
                windows_core::imp::DelegateBox::<TypedEventHandler<MediaPlayer, windows_core::IInspectable>, F>::new(
                    &TypedEventHandlerBox::<MediaPlayer, windows_core::IInspectable, F>::VTABLE,
                    value,
                );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).BufferingEnded)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveBufferingEnded,
            ))
        }
    }
    pub fn Play(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).Play)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn Pause(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).Pause)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn SetUriSource<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Uri>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetUriSource)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IMediaPlayer_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub AutoPlay: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetAutoPlay: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub NaturalDuration:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::TimeSpan) -> windows_core::HRESULT,
    pub Position:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::TimeSpan) -> windows_core::HRESULT,
    pub SetPosition: unsafe extern "system" fn(*mut core::ffi::c_void, windows_time::TimeSpan) -> windows_core::HRESULT,
    pub BufferingProgress: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub CurrentState: unsafe extern "system" fn(*mut core::ffi::c_void, *mut MediaPlayerState) -> windows_core::HRESULT,
    pub CanSeek: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub CanPause: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub IsLoopingEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsLoopingEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub IsProtected: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub IsMuted: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsMuted: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub PlaybackRate: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetPlaybackRate: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub Volume: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetVolume: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub PlaybackMediaMarkers:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub MediaOpened:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveMediaOpened: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub MediaEnded:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveMediaEnded: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub MediaFailed:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveMediaFailed: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub CurrentStateChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveCurrentStateChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PlaybackMediaMarkerReached:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePlaybackMediaMarkerReached:
        unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub MediaPlayerRateChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveMediaPlayerRateChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub VolumeChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveVolumeChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub SeekCompleted:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveSeekCompleted: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub BufferingStarted:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveBufferingStarted: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub BufferingEnded:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveBufferingEnded: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Play: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Pause: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetUriSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IMediaPlayer3, IMediaPlayer3_Vtbl, 0xee0660da_031b_4feb_bd9b_92e0a0a8d299);
impl windows_core::RuntimeType for IMediaPlayer3 {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IMediaPlayer3 {
    pub fn IsMutedChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, windows_core::IInspectable> = {
            let com =
                windows_core::imp::DelegateBox::<TypedEventHandler<MediaPlayer, windows_core::IInspectable>, F>::new(
                    &TypedEventHandlerBox::<MediaPlayer, windows_core::IInspectable, F>::VTABLE,
                    value,
                );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).IsMutedChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveIsMutedChanged,
            ))
        }
    }
    pub fn SourceChanged<F>(&self, value: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<MediaPlayer>, windows_core::Ref<windows_core::IInspectable>) + 'static,
    {
        let value: TypedEventHandler<MediaPlayer, windows_core::IInspectable> = {
            let com =
                windows_core::imp::DelegateBox::<TypedEventHandler<MediaPlayer, windows_core::IInspectable>, F>::new(
                    &TypedEventHandlerBox::<MediaPlayer, windows_core::IInspectable, F>::VTABLE,
                    value,
                );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).SourceChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&value),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveSourceChanged,
            ))
        }
    }
    pub fn AudioBalance(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AudioBalance)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetAudioBalance(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetAudioBalance)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn RealTimePlayback(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RealTimePlayback)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetRealTimePlayback(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetRealTimePlayback)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn StereoscopicVideoRenderMode(&self) -> windows_core::Result<StereoscopicVideoRenderMode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).StereoscopicVideoRenderMode)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetStereoscopicVideoRenderMode(&self, value: StereoscopicVideoRenderMode) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetStereoscopicVideoRenderMode)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn BreakManager(&self) -> windows_core::Result<MediaBreakManager> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BreakManager)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CommandManager(&self) -> windows_core::Result<MediaPlaybackCommandManager> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CommandManager)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn AudioDevice(&self) -> windows_core::Result<DeviceInformation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AudioDevice)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetAudioDevice<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<DeviceInformation>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetAudioDevice)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn TimelineController(&self) -> windows_core::Result<MediaTimelineController> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TimelineController)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetTimelineController<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<MediaTimelineController>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetTimelineController)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn TimelineControllerPositionOffset(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TimelineControllerPositionOffset)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetTimelineControllerPositionOffset(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTimelineControllerPositionOffset)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn PlaybackSession(&self) -> windows_core::Result<MediaPlaybackSession> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PlaybackSession)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn StepForwardOneFrame(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).StepForwardOneFrame)(windows_core::Interface::as_raw(self)).ok()
        }
    }
    pub fn StepBackwardOneFrame(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).StepBackwardOneFrame)(windows_core::Interface::as_raw(self)).ok()
        }
    }
    pub fn GetAsCastingSource(&self) -> windows_core::Result<CastingSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAsCastingSource)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IMediaPlayer3_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub IsMutedChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveIsMutedChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub SourceChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveSourceChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub AudioBalance: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetAudioBalance: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub RealTimePlayback: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetRealTimePlayback: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub StereoscopicVideoRenderMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut StereoscopicVideoRenderMode) -> windows_core::HRESULT,
    pub SetStereoscopicVideoRenderMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, StereoscopicVideoRenderMode) -> windows_core::HRESULT,
    pub BreakManager:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CommandManager:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub AudioDevice:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetAudioDevice:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub TimelineController:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetTimelineController:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub TimelineControllerPositionOffset:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::TimeSpan) -> windows_core::HRESULT,
    pub SetTimelineControllerPositionOffset:
        unsafe extern "system" fn(*mut core::ffi::c_void, windows_time::TimeSpan) -> windows_core::HRESULT,
    pub PlaybackSession:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub StepForwardOneFrame: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub StepBackwardOneFrame: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetAsCastingSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IMediaPlayerElement,
    IMediaPlayerElement_Vtbl,
    0x3b4ca526_2e25_4598_bc71_d491f8e8de39
);
impl windows_core::RuntimeType for IMediaPlayerElement {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IMediaPlayerElement {
    pub fn Source(&self) -> windows_core::Result<IMediaPlaybackSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Source)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetSource<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IMediaPlaybackSource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetSource)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn TransportControls(&self) -> windows_core::Result<MediaTransportControls> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TransportControls)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetTransportControls<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<MediaTransportControls>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetTransportControls)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn AreTransportControlsEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AreTransportControlsEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetAreTransportControlsEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetAreTransportControlsEnabled)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn PosterSource(&self) -> windows_core::Result<ImageSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PosterSource)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetPosterSource<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImageSource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetPosterSource)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Stretch(&self) -> windows_core::Result<Stretch> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Stretch)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetStretch(&self, value: Stretch) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetStretch)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn AutoPlay(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AutoPlay)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetAutoPlay(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetAutoPlay)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn IsFullWindow(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsFullWindow)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetIsFullWindow(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsFullWindow)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn MediaPlayer(&self) -> windows_core::Result<MediaPlayer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MediaPlayer)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetMediaPlayer<P0>(&self, mediaplayer: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<MediaPlayer>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetMediaPlayer)(
                windows_core::Interface::as_raw(self),
                mediaplayer.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IMediaPlayerElement_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Source: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetSource: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub TransportControls:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetTransportControls:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub AreTransportControlsEnabled:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetAreTransportControlsEnabled:
        unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub PosterSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetPosterSource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Stretch: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Stretch) -> windows_core::HRESULT,
    pub SetStretch: unsafe extern "system" fn(*mut core::ffi::c_void, Stretch) -> windows_core::HRESULT,
    pub AutoPlay: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetAutoPlay: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub IsFullWindow: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsFullWindow: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub MediaPlayer:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetMediaPlayer:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IMediaPlayerFailedEventArgs,
    IMediaPlayerFailedEventArgs_Vtbl,
    0x2744e9b9_a7e3_4f16_bac4_7914ebc08301
);
impl windows_core::RuntimeType for IMediaPlayerFailedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IMediaPlayerFailedEventArgs {
    pub fn Error(&self) -> windows_core::Result<MediaPlayerError> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Error)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn ExtendedErrorCode(&self) -> windows_core::Result<windows_core::HRESULT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ExtendedErrorCode)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ErrorMessage(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ErrorMessage)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                })
        }
    }
}
#[repr(C)]
pub struct IMediaPlayerFailedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Error: unsafe extern "system" fn(*mut core::ffi::c_void, *mut MediaPlayerError) -> windows_core::HRESULT,
    pub ExtendedErrorCode:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_core::HRESULT) -> windows_core::HRESULT,
    pub ErrorMessage:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IMediaPlayerRateChangedEventArgs,
    IMediaPlayerRateChangedEventArgs_Vtbl,
    0x40600d58_3b61_4bb2_989f_fc65608b6cab
);
impl windows_core::RuntimeType for IMediaPlayerRateChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaPlayerRateChangedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IMediaPlayerSource2,
    IMediaPlayerSource2_Vtbl,
    0x82449b9f_7322_4c0b_b03b_3e69a48260c5
);
impl windows_core::RuntimeType for IMediaPlayerSource2 {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IMediaPlayerSource2 {
    pub fn Source(&self) -> windows_core::Result<IMediaPlaybackSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Source)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetSource<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IMediaPlaybackSource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetSource)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IMediaPlayerSource2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Source: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetSource: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IMediaSource, IMediaSource_Vtbl, 0xe7bfb599_a09d_4c21_bcdf_20af4f86b3d9);
impl windows_core::RuntimeType for IMediaSource {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IMediaSource, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct IMediaSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IMediaSource2, IMediaSource2_Vtbl, 0x2eb61048_655f_4c37_b813_b4e45dfa0abe);
impl windows_core::RuntimeType for IMediaSource2 {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaSource2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IMediaSourceStatics,
    IMediaSourceStatics_Vtbl,
    0xf77d6fa4_4652_410e_b1d8_e9a5e245a45c
);
impl windows_core::RuntimeType for IMediaSourceStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaSourceStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateFromAdaptiveMediaSource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFromMediaStreamSource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFromMseStreamSource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFromIMediaSource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFromStorageFile: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFromStream: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFromStreamReference: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFromUri: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IMediaSourceStatics2,
    IMediaSourceStatics2_Vtbl,
    0xeee161a4_7f13_4896_b8cb_df0de5bcb9f1
);
impl windows_core::RuntimeType for IMediaSourceStatics2 {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaSourceStatics2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateFromMediaBinder: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IMediaStreamSource,
    IMediaStreamSource_Vtbl,
    0x3712d543_45eb_4138_aa62_c01e26f3843f
);
impl windows_core::RuntimeType for IMediaStreamSource {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaStreamSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IMediaTimelineController,
    IMediaTimelineController_Vtbl,
    0x8ed361f3_0b78_4360_bf71_0c841999ea1b
);
impl windows_core::RuntimeType for IMediaTimelineController {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaTimelineController_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IMediaTrack, IMediaTrack_Vtbl, 0x03e1fafc_c931_491a_b46b_c10ee8c256b7);
impl windows_core::RuntimeType for IMediaTrack {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IMediaTrack, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct IMediaTrack_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IMediaTransportControls,
    IMediaTransportControls_Vtbl,
    0xd6f69e7d_0825_49a9_9fce_5586d8694f0c
);
impl windows_core::RuntimeType for IMediaTransportControls {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMediaTransportControls_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IMenuFlyout, IMenuFlyout_Vtbl, 0xad3210f1_985b_4171_bbba_67bcc729649b);
impl windows_core::RuntimeType for IMenuFlyout {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IMenuFlyout {
    pub fn Items(&self) -> windows_core::Result<IVector<MenuFlyoutItemBase>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Items)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn MenuFlyoutPresenterStyle(&self) -> windows_core::Result<Style> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MenuFlyoutPresenterStyle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetMenuFlyoutPresenterStyle<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Style>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetMenuFlyoutPresenterStyle)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IMenuFlyout_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Items: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub MenuFlyoutPresenterStyle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetMenuFlyoutPresenterStyle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IMenuFlyoutItem, IMenuFlyoutItem_Vtbl, 0x160775cf_b598_44af_a488_c3ce2af6d3f0);
impl windows_core::RuntimeType for IMenuFlyoutItem {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IMenuFlyoutItem {
    pub fn Text(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Text)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn SetText(&self, value: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetText)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
            )
            .ok()
        }
    }
    pub fn Command(&self) -> windows_core::Result<ICommand> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Command)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetCommand<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICommand>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetCommand)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn CommandParameter(&self) -> windows_core::Result<windows_core::IInspectable> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CommandParameter)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetCommandParameter<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetCommandParameter)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Click<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
    {
        let handler: RoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RoutedEventHandler, F>::new(
                &RoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Click)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveClick,
            ))
        }
    }
}
#[repr(C)]
pub struct IMenuFlyoutItem_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Text: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetText: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Command:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetCommand: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CommandParameter:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetCommandParameter:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Click:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveClick: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IMenuFlyoutItemBase,
    IMenuFlyoutItemBase_Vtbl,
    0xf9b56047_8da6_4730_9208_144125c6b396
);
impl windows_core::RuntimeType for IMenuFlyoutItemBase {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMenuFlyoutItemBase_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IMseStreamSource, IMseStreamSource_Vtbl, 0xb0b4198d_02f4_4923_88dd_81bc3f360ffa);
impl windows_core::RuntimeType for IMseStreamSource {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IMseStreamSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IOutputStream, IOutputStream_Vtbl, 0x905a0fe6_bc53_11df_8c49_001e4fc686da);
impl windows_core::RuntimeType for IOutputStream {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IOutputStream, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(IOutputStream, IClosable);
impl IOutputStream {
    pub fn WriteAsync<P0>(
        &self,
        buffer: P0,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<u32, u32>>
    where
        P0: windows_core::Param<IBuffer>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).WriteAsync)(
                windows_core::Interface::as_raw(self),
                buffer.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn FlushAsync(&self) -> windows_core::Result<windows_future::IAsyncOperation<bool>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FlushAsync)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IOutputStream_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub WriteAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FlushAsync:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IPanel, IPanel_Vtbl, 0xa50a4bbd_8361_469c_90da_e9a40c7474df);
impl windows_core::RuntimeType for IPanel {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IPanel {
    pub fn Children(&self) -> windows_core::Result<UIElementCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Children)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Background(&self) -> windows_core::Result<Brush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Background)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetBackground<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetBackground)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn IsItemsHost(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsItemsHost)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn ChildrenTransitions(&self) -> windows_core::Result<TransitionCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ChildrenTransitions)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetChildrenTransitions<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<TransitionCollection>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetChildrenTransitions)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IPanel_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Children:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Background:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetBackground:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub IsItemsHost: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub ChildrenTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetChildrenTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IPasswordCredential,
    IPasswordCredential_Vtbl,
    0x6ab18989_c720_41a7_a6c1_feadb36329a0
);
impl windows_core::RuntimeType for IPasswordCredential {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IPasswordCredential {
    pub fn Resource(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Resource)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn SetResource(&self, resource: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetResource)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(resource)),
            )
            .ok()
        }
    }
    pub fn UserName(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).UserName)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn SetUserName(&self, username: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetUserName)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(username)),
            )
            .ok()
        }
    }
    pub fn Password(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Password)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn SetPassword(&self, password: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetPassword)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(password)),
            )
            .ok()
        }
    }
    pub fn RetrievePassword(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).RetrievePassword)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn Properties(&self) -> windows_core::Result<IPropertySet> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Properties)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IPasswordCredential_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Resource:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetResource: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub UserName:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetUserName: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Password:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetPassword: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub RetrievePassword: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Properties:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IPasswordVault, IPasswordVault_Vtbl, 0x61fd2c0b_c8d4_48c1_a54f_bc5a64205af2);
impl windows_core::RuntimeType for IPasswordVault {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IPasswordVault {
    pub fn Add<P0>(&self, credential: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<PasswordCredential>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Add)(windows_core::Interface::as_raw(self), credential.param().abi())
                .ok()
        }
    }
    pub fn Remove<P0>(&self, credential: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<PasswordCredential>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Remove)(
                windows_core::Interface::as_raw(self),
                credential.param().abi(),
            )
            .ok()
        }
    }
    pub fn Retrieve(&self, resource: &str, username: &str) -> windows_core::Result<PasswordCredential> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Retrieve)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(resource)),
                core::mem::transmute_copy(&windows_core::HSTRING::from(username)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn FindAllByResource(
        &self,
        resource: &str,
    ) -> windows_core::Result<windows_collections::IVectorView<PasswordCredential>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindAllByResource)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(resource)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn FindAllByUserName(
        &self,
        username: &str,
    ) -> windows_core::Result<windows_collections::IVectorView<PasswordCredential>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindAllByUserName)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(username)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn RetrieveAll(&self) -> windows_core::Result<windows_collections::IVectorView<PasswordCredential>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RetrieveAll)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IPasswordVault_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Add: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Remove: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Retrieve: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindAllByResource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindAllByUserName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RetrieveAll:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IPlayToSource, IPlayToSource_Vtbl, 0x7f138a08_fbb7_4b09_8356_aa5f4e335c31);
impl windows_core::RuntimeType for IPlayToSource {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IPlayToSource_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IPlaybackMediaMarkerReachedEventArgs,
    IPlaybackMediaMarkerReachedEventArgs_Vtbl,
    0x578cd1b9_90e2_4e60_abc4_8740b01f6196
);
impl windows_core::RuntimeType for IPlaybackMediaMarkerReachedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IPlaybackMediaMarkerReachedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IPlaybackMediaMarkerSequence,
    IPlaybackMediaMarkerSequence_Vtbl,
    0xf2810cee_638b_46cf_8817_1d111fe9d8c4
);
impl windows_core::RuntimeType for IPlaybackMediaMarkerSequence {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IPlaybackMediaMarkerSequence_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IPointer, IPointer_Vtbl, 0x5ee8f39f_747d_4171_90e6_cd37a9dffb11);
impl windows_core::RuntimeType for IPointer {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IPointer_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IPointerRoutedEventArgs,
    IPointerRoutedEventArgs_Vtbl,
    0xda628f0a_9752_49e2_bde2_49eccab9194d
);
impl windows_core::RuntimeType for IPointerRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IPointerRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IProjection, IProjection_Vtbl, 0xb3443557_7f39_4d04_a89c_844338cac897);
impl windows_core::RuntimeType for IProjection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IProjection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IPropertySet, IPropertySet_Vtbl, 0x8a43ed9f_f4e6_4421_acf9_1dab2986820c);
impl windows_core::RuntimeType for IPropertySet {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IPropertySet, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct IPropertySet_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IPropertyValue, IPropertyValue_Vtbl, 0x4bd682dd_7554_40e9_9a9b_82654ede7e62);
impl windows_core::RuntimeType for IPropertyValue {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IPropertyValue, windows_core::IUnknown, windows_core::IInspectable);
impl IPropertyValue {
    pub fn Type(&self) -> windows_core::Result<PropertyType> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Type)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn IsNumericScalar(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsNumericScalar)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn GetUInt8(&self) -> windows_core::Result<u8> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetUInt8)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetInt16(&self) -> windows_core::Result<i16> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetInt16)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetUInt16(&self) -> windows_core::Result<u16> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetUInt16)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetInt32(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetInt32)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetUInt32(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetUInt32)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetInt64(&self) -> windows_core::Result<i64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetInt64)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetUInt64(&self) -> windows_core::Result<u64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetUInt64)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetSingle(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetSingle)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetDouble(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetDouble)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetChar16(&self) -> windows_core::Result<u16> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetChar16)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetBoolean(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetBoolean)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetString(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetString)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn GetGuid(&self) -> windows_core::Result<windows_core::GUID> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetGuid)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetDateTime(&self) -> windows_core::Result<windows_time::DateTime> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetDateTime)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetTimeSpan(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetTimeSpan)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetPoint(&self) -> windows_core::Result<Point> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPoint)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetSize(&self) -> windows_core::Result<Size> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetSize)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetRect(&self) -> windows_core::Result<Rect> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetRect)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetUInt8Array(&self, value: &mut windows_core::Array<u8>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetUInt8Array)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetInt16Array(&self, value: &mut windows_core::Array<i16>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetInt16Array)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetUInt16Array(&self, value: &mut windows_core::Array<u16>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetUInt16Array)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetInt32Array(&self, value: &mut windows_core::Array<i32>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetInt32Array)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetUInt32Array(&self, value: &mut windows_core::Array<u32>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetUInt32Array)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetInt64Array(&self, value: &mut windows_core::Array<i64>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetInt64Array)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetUInt64Array(&self, value: &mut windows_core::Array<u64>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetUInt64Array)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetSingleArray(&self, value: &mut windows_core::Array<f32>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetSingleArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetDoubleArray(&self, value: &mut windows_core::Array<f64>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetDoubleArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetChar16Array(&self, value: &mut windows_core::Array<u16>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetChar16Array)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetBooleanArray(&self, value: &mut windows_core::Array<bool>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetBooleanArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetStringArray(&self, value: &mut windows_core::Array<windows_core::HSTRING>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetStringArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetInspectableArray(
        &self,
        value: &mut windows_core::Array<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetInspectableArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetGuidArray(&self, value: &mut windows_core::Array<windows_core::GUID>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetGuidArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetDateTimeArray(
        &self,
        value: &mut windows_core::Array<windows_time::DateTime>,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetDateTimeArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetTimeSpanArray(
        &self,
        value: &mut windows_core::Array<windows_time::TimeSpan>,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetTimeSpanArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetPointArray(&self, value: &mut windows_core::Array<Point>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetPointArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetSizeArray(&self, value: &mut windows_core::Array<Size>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetSizeArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
    pub fn GetRectArray(&self, value: &mut windows_core::Array<Rect>) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).GetRectArray)(
                windows_core::Interface::as_raw(self),
                value.set_abi_len(),
                value as *mut _ as _,
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IPropertyValue_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Type: unsafe extern "system" fn(*mut core::ffi::c_void, *mut PropertyType) -> windows_core::HRESULT,
    pub IsNumericScalar: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub GetUInt8: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u8) -> windows_core::HRESULT,
    pub GetInt16: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i16) -> windows_core::HRESULT,
    pub GetUInt16: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u16) -> windows_core::HRESULT,
    pub GetInt32: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub GetUInt32: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub GetInt64: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub GetUInt64: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u64) -> windows_core::HRESULT,
    pub GetSingle: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub GetDouble: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub GetChar16: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u16) -> windows_core::HRESULT,
    pub GetBoolean: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub GetString:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetGuid: unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_core::GUID) -> windows_core::HRESULT,
    pub GetDateTime:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::DateTime) -> windows_core::HRESULT,
    pub GetTimeSpan:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::TimeSpan) -> windows_core::HRESULT,
    pub GetPoint: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Point) -> windows_core::HRESULT,
    pub GetSize: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Size) -> windows_core::HRESULT,
    pub GetRect: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Rect) -> windows_core::HRESULT,
    pub GetUInt8Array:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut u8) -> windows_core::HRESULT,
    pub GetInt16Array:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut i16) -> windows_core::HRESULT,
    pub GetUInt16Array:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut u16) -> windows_core::HRESULT,
    pub GetInt32Array:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut i32) -> windows_core::HRESULT,
    pub GetUInt32Array:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut u32) -> windows_core::HRESULT,
    pub GetInt64Array:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut i64) -> windows_core::HRESULT,
    pub GetUInt64Array:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut u64) -> windows_core::HRESULT,
    pub GetSingleArray:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut f32) -> windows_core::HRESULT,
    pub GetDoubleArray:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut f64) -> windows_core::HRESULT,
    pub GetChar16Array:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut u16) -> windows_core::HRESULT,
    pub GetBooleanArray:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut bool) -> windows_core::HRESULT,
    pub GetStringArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut u32,
        *mut *mut windows_core::HSTRING,
    ) -> windows_core::HRESULT,
    pub GetInspectableArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut u32,
        *mut *mut windows_core::IInspectable,
    ) -> windows_core::HRESULT,
    pub GetGuidArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut u32,
        *mut *mut windows_core::GUID,
    ) -> windows_core::HRESULT,
    pub GetDateTimeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut u32,
        *mut *mut windows_time::DateTime,
    ) -> windows_core::HRESULT,
    pub GetTimeSpanArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut u32,
        *mut *mut windows_time::TimeSpan,
    ) -> windows_core::HRESULT,
    pub GetPointArray:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut Point) -> windows_core::HRESULT,
    pub GetSizeArray:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut Size) -> windows_core::HRESULT,
    pub GetRectArray:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32, *mut *mut Rect) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IPropertyValueStatics,
    IPropertyValueStatics_Vtbl,
    0x629bdbc8_d932_4ff4_96b9_8d96c5c1e858
);
impl windows_core::RuntimeType for IPropertyValueStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IPropertyValueStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateEmpty:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateUInt8:
        unsafe extern "system" fn(*mut core::ffi::c_void, u8, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateInt16:
        unsafe extern "system" fn(*mut core::ffi::c_void, i16, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateUInt16:
        unsafe extern "system" fn(*mut core::ffi::c_void, u16, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateInt32:
        unsafe extern "system" fn(*mut core::ffi::c_void, i32, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateUInt32:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateInt64:
        unsafe extern "system" fn(*mut core::ffi::c_void, i64, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateUInt64:
        unsafe extern "system" fn(*mut core::ffi::c_void, u64, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateSingle:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateDouble:
        unsafe extern "system" fn(*mut core::ffi::c_void, f64, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateChar16:
        unsafe extern "system" fn(*mut core::ffi::c_void, u16, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateBoolean:
        unsafe extern "system" fn(*mut core::ffi::c_void, bool, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateString: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateInspectable: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateGuid: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateDateTime: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_time::DateTime,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateTimeSpan: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_time::TimeSpan,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePoint:
        unsafe extern "system" fn(*mut core::ffi::c_void, Point, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateSize:
        unsafe extern "system" fn(*mut core::ffi::c_void, Size, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateRect:
        unsafe extern "system" fn(*mut core::ffi::c_void, Rect, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CreateUInt8Array: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const u8,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateInt16Array: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const i16,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateUInt16Array: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const u16,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateInt32Array: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateUInt32Array: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateInt64Array: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const i64,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateUInt64Array: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const u64,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSingleArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const f32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateDoubleArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const f64,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateChar16Array: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const u16,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBooleanArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const bool,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateStringArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const windows_core::HSTRING,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateInspectableArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const windows_core::IInspectable,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateGuidArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateDateTimeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const windows_time::DateTime,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateTimeSpanArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const windows_time::TimeSpan,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePointArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const Point,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSizeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const Size,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateRectArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const Rect,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IRandomAccessStream,
    IRandomAccessStream_Vtbl,
    0x905a0fe1_bc53_11df_8c49_001e4fc686da
);
impl windows_core::RuntimeType for IRandomAccessStream {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IRandomAccessStream, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(IRandomAccessStream, IClosable, IInputStream, IOutputStream);
impl IRandomAccessStream {
    pub fn Size(&self) -> windows_core::Result<u64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Size)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetSize(&self, value: u64) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetSize)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn GetInputStreamAt(&self, position: u64) -> windows_core::Result<IInputStream> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetInputStreamAt)(
                windows_core::Interface::as_raw(self),
                position,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetOutputStreamAt(&self, position: u64) -> windows_core::Result<IOutputStream> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetOutputStreamAt)(
                windows_core::Interface::as_raw(self),
                position,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Position(&self) -> windows_core::Result<u64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Position)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn Seek(&self, position: u64) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).Seek)(windows_core::Interface::as_raw(self), position).ok() }
    }
    pub fn CloneStream(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CloneStream)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CanRead(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CanRead)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn CanWrite(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CanWrite)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IRandomAccessStream_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Size: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u64) -> windows_core::HRESULT,
    pub SetSize: unsafe extern "system" fn(*mut core::ffi::c_void, u64) -> windows_core::HRESULT,
    pub GetInputStreamAt:
        unsafe extern "system" fn(*mut core::ffi::c_void, u64, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetOutputStreamAt:
        unsafe extern "system" fn(*mut core::ffi::c_void, u64, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Position: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u64) -> windows_core::HRESULT,
    pub Seek: unsafe extern "system" fn(*mut core::ffi::c_void, u64) -> windows_core::HRESULT,
    pub CloneStream:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CanRead: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub CanWrite: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IRandomAccessStreamReference,
    IRandomAccessStreamReference_Vtbl,
    0x33ee3134_1dd6_4e3a_8067_d1c162e8642b
);
impl windows_core::RuntimeType for IRandomAccessStreamReference {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(
    IRandomAccessStreamReference,
    windows_core::IUnknown,
    windows_core::IInspectable
);
#[repr(C)]
pub struct IRandomAccessStreamReference_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IRectangleGeometry,
    IRectangleGeometry_Vtbl,
    0xa25a1f58_c575_4196_91cf_9fdfb10445c3
);
impl windows_core::RuntimeType for IRectangleGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IRectangleGeometry {
    pub fn Rect(&self) -> windows_core::Result<Rect> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Rect)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetRect(&self, value: Rect) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetRect)(windows_core::Interface::as_raw(self), value).ok() }
    }
}
#[repr(C)]
pub struct IRectangleGeometry_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Rect: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Rect) -> windows_core::HRESULT,
    pub SetRect: unsafe extern "system" fn(*mut core::ffi::c_void, Rect) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IResourceDictionary,
    IResourceDictionary_Vtbl,
    0xc1ea4f24_d6de_4191_8e3a_f48601f7489c
);
impl windows_core::RuntimeType for IResourceDictionary {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IResourceDictionary_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IRightTappedRoutedEventArgs,
    IRightTappedRoutedEventArgs_Vtbl,
    0x6834869d_7bd5_4033_b237_172f79abe393
);
impl windows_core::RuntimeType for IRightTappedRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IRightTappedRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IRoutedEvent, IRoutedEvent_Vtbl, 0xa6b25818_43c1_4c70_865c_7bdd5a32e327);
impl windows_core::RuntimeType for IRoutedEvent {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IRoutedEvent_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IRoutedEventArgs, IRoutedEventArgs_Vtbl, 0x5c985ac6_d802_4b38_a223_bf070c43fedf);
impl windows_core::RuntimeType for IRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IRowDefinition, IRowDefinition_Vtbl, 0x4abae829_d80c_4a5e_a48c_f8b3d3b6533d);
impl windows_core::RuntimeType for IRowDefinition {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IRowDefinition {
    pub fn Height(&self) -> windows_core::Result<GridLength> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Height)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetHeight(&self, value: GridLength) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetHeight)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn MaxHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MaxHeight)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetMaxHeight(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMaxHeight)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn MinHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MinHeight)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetMinHeight(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMinHeight)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn ActualHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ActualHeight)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IRowDefinition_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Height: unsafe extern "system" fn(*mut core::ffi::c_void, *mut GridLength) -> windows_core::HRESULT,
    pub SetHeight: unsafe extern "system" fn(*mut core::ffi::c_void, GridLength) -> windows_core::HRESULT,
    pub MaxHeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetMaxHeight: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub MinHeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetMinHeight: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub ActualHeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ISearchActivatedEventArgs,
    ISearchActivatedEventArgs_Vtbl,
    0x8cb36951_58c8_43e3_94bc_41d33f8b630e
);
impl windows_core::RuntimeType for ISearchActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(ISearchActivatedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct ISearchActivatedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(ISelector, ISelector_Vtbl, 0xe30eb3a5_b36b_42dc_8527_cd25136c083c);
impl windows_core::RuntimeType for ISelector {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ISelector_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(ISelectorItem, ISelectorItem_Vtbl, 0x541c8d6c_0283_4581_b945_2a64c28a0646);
impl windows_core::RuntimeType for ISelectorItem {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ISelectorItem_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ISetVersionRequest,
    ISetVersionRequest_Vtbl,
    0xb9c76b9b_1056_4e69_8330_162619956f9b
);
impl windows_core::RuntimeType for ISetVersionRequest {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ISetVersionRequest_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IShareTargetActivatedEventArgs,
    IShareTargetActivatedEventArgs_Vtbl,
    0x4bdaf9c8_cdb2_4acb_bfc3_6648563378ec
);
impl windows_core::RuntimeType for IShareTargetActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(
    IShareTargetActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
#[repr(C)]
pub struct IShareTargetActivatedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ISizeChangedEventArgs,
    ISizeChangedEventArgs_Vtbl,
    0xd5312e60_5cc1_42a1_920c_1af46be2f986
);
impl windows_core::RuntimeType for ISizeChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ISizeChangedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(ISolidColorBrush, ISolidColorBrush_Vtbl, 0x9d850850_66f3_48df_9a8f_824bd5e070af);
impl windows_core::RuntimeType for ISolidColorBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ISolidColorBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ISolidColorBrushFactory,
    ISolidColorBrushFactory_Vtbl,
    0xd935ce0c_86f5_4da6_8a27_b1619ef7f92b
);
impl windows_core::RuntimeType for ISolidColorBrushFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ISolidColorBrushFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateInstanceWithColor:
        unsafe extern "system" fn(*mut core::ffi::c_void, Color, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IStorageFile, IStorageFile_Vtbl, 0xfa3f6186_4214_428c_a64c_14c9ac7315ea);
impl windows_core::RuntimeType for IStorageFile {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IStorageFile, windows_core::IUnknown, windows_core::IInspectable);
#[repr(C)]
pub struct IStorageFile_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IStorageFolder, IStorageFolder_Vtbl, 0x72d1cb78_b3ef_4f75_a80b_6fd9dae2944b);
impl windows_core::RuntimeType for IStorageFolder {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IStorageFolder, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(IStorageFolder, IStorageItem);
impl IStorageFolder {
    pub fn CreateFileAsyncOverloadDefaultOptions(
        &self,
        desiredname: &str,
    ) -> windows_core::Result<windows_future::IAsyncOperation<StorageFile>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFileAsyncOverloadDefaultOptions)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(desiredname)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CreateFileAsync(
        &self,
        desiredname: &str,
        options: CreationCollisionOption,
    ) -> windows_core::Result<windows_future::IAsyncOperation<StorageFile>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFileAsync)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(desiredname)),
                options,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CreateFolderAsyncOverloadDefaultOptions(
        &self,
        desiredname: &str,
    ) -> windows_core::Result<windows_future::IAsyncOperation<StorageFolder>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFolderAsyncOverloadDefaultOptions)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(desiredname)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn CreateFolderAsync(
        &self,
        desiredname: &str,
        options: CreationCollisionOption,
    ) -> windows_core::Result<windows_future::IAsyncOperation<StorageFolder>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFolderAsync)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(desiredname)),
                options,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetFileAsync(&self, name: &str) -> windows_core::Result<windows_future::IAsyncOperation<StorageFile>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFileAsync)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(name)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetFolderAsync(&self, name: &str) -> windows_core::Result<windows_future::IAsyncOperation<StorageFolder>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFolderAsync)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(name)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetItemAsync(&self, name: &str) -> windows_core::Result<windows_future::IAsyncOperation<IStorageItem>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetItemAsync)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(name)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetFilesAsyncOverloadDefaultOptionsStartAndCount(
        &self,
    ) -> windows_core::Result<windows_future::IAsyncOperation<windows_collections::IVectorView<StorageFile>>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFilesAsyncOverloadDefaultOptionsStartAndCount)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetFoldersAsyncOverloadDefaultOptionsStartAndCount(
        &self,
    ) -> windows_core::Result<windows_future::IAsyncOperation<windows_collections::IVectorView<StorageFolder>>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFoldersAsyncOverloadDefaultOptionsStartAndCount)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetItemsAsyncOverloadDefaultStartAndCount(
        &self,
    ) -> windows_core::Result<windows_future::IAsyncOperation<windows_collections::IVectorView<IStorageItem>>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetItemsAsyncOverloadDefaultStartAndCount)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IStorageFolder_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateFileAsyncOverloadDefaultOptions: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFileAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        CreationCollisionOption,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFolderAsyncOverloadDefaultOptions: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFolderAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        CreationCollisionOption,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetFileAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetFolderAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetItemAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetFilesAsyncOverloadDefaultOptionsStartAndCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetFoldersAsyncOverloadDefaultOptionsStartAndCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetItemsAsyncOverloadDefaultStartAndCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IStorageItem, IStorageItem_Vtbl, 0x4207a996_ca2f_42f7_bde8_8b10457a7f30);
impl windows_core::RuntimeType for IStorageItem {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
windows_core::imp::interface_hierarchy!(IStorageItem, windows_core::IUnknown, windows_core::IInspectable);
impl IStorageItem {
    pub fn RenameAsyncOverloadDefaultOptions(&self, desiredname: &str) -> windows_core::Result<IAsyncAction> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RenameAsyncOverloadDefaultOptions)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(desiredname)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn RenameAsync(&self, desiredname: &str, option: NameCollisionOption) -> windows_core::Result<IAsyncAction> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RenameAsync)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(desiredname)),
                option,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn DeleteAsyncOverloadDefaultOptions(&self) -> windows_core::Result<IAsyncAction> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DeleteAsyncOverloadDefaultOptions)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn DeleteAsync(&self, option: StorageDeleteOption) -> windows_core::Result<IAsyncAction> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DeleteAsync)(
                windows_core::Interface::as_raw(self),
                option,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn GetBasicPropertiesAsync(&self) -> windows_core::Result<windows_future::IAsyncOperation<BasicProperties>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetBasicPropertiesAsync)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Name(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Name)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn Path(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Path)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn Attributes(&self) -> windows_core::Result<FileAttributes> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Attributes)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn DateCreated(&self) -> windows_core::Result<windows_time::DateTime> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DateCreated)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn IsOfType(&self, r#type: StorageItemTypes) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsOfType)(
                windows_core::Interface::as_raw(self),
                r#type,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IStorageItem_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub RenameAsyncOverloadDefaultOptions: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RenameAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        NameCollisionOption,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DeleteAsyncOverloadDefaultOptions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DeleteAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        StorageDeleteOption,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetBasicPropertiesAsync:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Name: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Path: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Attributes: unsafe extern "system" fn(*mut core::ffi::c_void, *mut FileAttributes) -> windows_core::HRESULT,
    pub DateCreated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_time::DateTime) -> windows_core::HRESULT,
    pub IsOfType:
        unsafe extern "system" fn(*mut core::ffi::c_void, StorageItemTypes, *mut bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IStyle, IStyle_Vtbl, 0xc4a9f225_9db7_4a7d_b6d1_f74edb9293c2);
impl windows_core::RuntimeType for IStyle {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IStyle_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IStyleSelector, IStyleSelector_Vtbl, 0xd1113f67_d8c1_4ae4_98f0_d8504502f08b);
impl windows_core::RuntimeType for IStyleSelector {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IStyleSelector_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ITappedRoutedEventArgs,
    ITappedRoutedEventArgs_Vtbl,
    0xa099e6be_e624_459a_bb1d_e05c73e2cc66
);
impl windows_core::RuntimeType for ITappedRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ITappedRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(ITextBlock, ITextBlock_Vtbl, 0xae2d9271_3b4a_45fc_8468_f7949548f4d5);
impl windows_core::RuntimeType for ITextBlock {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl ITextBlock {
    pub fn FontSize(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FontSize)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetFontSize(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetFontSize)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn FontFamily(&self) -> windows_core::Result<FontFamily> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FontFamily)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetFontFamily<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<FontFamily>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetFontFamily)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn FontWeight(&self) -> windows_core::Result<FontWeight> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FontWeight)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetFontWeight(&self, value: FontWeight) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetFontWeight)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn FontStyle(&self) -> windows_core::Result<FontStyle> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FontStyle)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetFontStyle(&self, value: FontStyle) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetFontStyle)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn FontStretch(&self) -> windows_core::Result<FontStretch> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FontStretch)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetFontStretch(&self, value: FontStretch) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetFontStretch)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn CharacterSpacing(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CharacterSpacing)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetCharacterSpacing(&self, value: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetCharacterSpacing)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn Foreground(&self) -> windows_core::Result<Brush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Foreground)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetForeground<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetForeground)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn TextWrapping(&self) -> windows_core::Result<TextWrapping> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TextWrapping)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetTextWrapping(&self, value: TextWrapping) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTextWrapping)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn TextTrimming(&self) -> windows_core::Result<TextTrimming> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TextTrimming)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetTextTrimming(&self, value: TextTrimming) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTextTrimming)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn TextAlignment(&self) -> windows_core::Result<TextAlignment> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TextAlignment)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetTextAlignment(&self, value: TextAlignment) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTextAlignment)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn Text(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Text)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn SetText(&self, value: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetText)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
            )
            .ok()
        }
    }
    pub fn Inlines(&self) -> windows_core::Result<InlineCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Inlines)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Padding(&self) -> windows_core::Result<Thickness> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Padding)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetPadding(&self, value: Thickness) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetPadding)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn LineHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LineHeight)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetLineHeight(&self, value: f64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetLineHeight)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn LineStackingStrategy(&self) -> windows_core::Result<LineStackingStrategy> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LineStackingStrategy)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetLineStackingStrategy(&self, value: LineStackingStrategy) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetLineStackingStrategy)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn IsTextSelectionEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsTextSelectionEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIsTextSelectionEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsTextSelectionEnabled)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn SelectedText(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SelectedText)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                })
        }
    }
    pub fn ContentStart(&self) -> windows_core::Result<TextPointer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentStart)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn ContentEnd(&self) -> windows_core::Result<TextPointer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentEnd)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SelectionStart(&self) -> windows_core::Result<TextPointer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SelectionStart)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SelectionEnd(&self) -> windows_core::Result<TextPointer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SelectionEnd)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn BaselineOffset(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BaselineOffset)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SelectionChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
    {
        let handler: RoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RoutedEventHandler, F>::new(
                &RoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).SelectionChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveSelectionChanged,
            ))
        }
    }
    pub fn ContextMenuOpening<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ContextMenuEventArgs>) + 'static,
    {
        let handler: ContextMenuOpeningEventHandler = {
            let com = windows_core::imp::DelegateBox::<ContextMenuOpeningEventHandler, F>::new(
                &ContextMenuOpeningEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ContextMenuOpening)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveContextMenuOpening,
            ))
        }
    }
    pub fn SelectAll(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SelectAll)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn Select<P0, P1>(&self, start: P0, end: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<TextPointer>,
        P1: windows_core::Param<TextPointer>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Select)(
                windows_core::Interface::as_raw(self),
                start.param().abi(),
                end.param().abi(),
            )
            .ok()
        }
    }
    pub fn Focus(&self, value: FocusState) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Focus)(windows_core::Interface::as_raw(self), value, &mut result__)
                .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct ITextBlock_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub FontSize: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetFontSize: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub FontFamily:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetFontFamily:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub FontWeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut FontWeight) -> windows_core::HRESULT,
    pub SetFontWeight: unsafe extern "system" fn(*mut core::ffi::c_void, FontWeight) -> windows_core::HRESULT,
    pub FontStyle: unsafe extern "system" fn(*mut core::ffi::c_void, *mut FontStyle) -> windows_core::HRESULT,
    pub SetFontStyle: unsafe extern "system" fn(*mut core::ffi::c_void, FontStyle) -> windows_core::HRESULT,
    pub FontStretch: unsafe extern "system" fn(*mut core::ffi::c_void, *mut FontStretch) -> windows_core::HRESULT,
    pub SetFontStretch: unsafe extern "system" fn(*mut core::ffi::c_void, FontStretch) -> windows_core::HRESULT,
    pub CharacterSpacing: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetCharacterSpacing: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub Foreground:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetForeground:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub TextWrapping: unsafe extern "system" fn(*mut core::ffi::c_void, *mut TextWrapping) -> windows_core::HRESULT,
    pub SetTextWrapping: unsafe extern "system" fn(*mut core::ffi::c_void, TextWrapping) -> windows_core::HRESULT,
    pub TextTrimming: unsafe extern "system" fn(*mut core::ffi::c_void, *mut TextTrimming) -> windows_core::HRESULT,
    pub SetTextTrimming: unsafe extern "system" fn(*mut core::ffi::c_void, TextTrimming) -> windows_core::HRESULT,
    pub TextAlignment: unsafe extern "system" fn(*mut core::ffi::c_void, *mut TextAlignment) -> windows_core::HRESULT,
    pub SetTextAlignment: unsafe extern "system" fn(*mut core::ffi::c_void, TextAlignment) -> windows_core::HRESULT,
    pub Text: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetText: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Inlines:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Padding: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Thickness) -> windows_core::HRESULT,
    pub SetPadding: unsafe extern "system" fn(*mut core::ffi::c_void, Thickness) -> windows_core::HRESULT,
    pub LineHeight: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetLineHeight: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub LineStackingStrategy:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut LineStackingStrategy) -> windows_core::HRESULT,
    pub SetLineStackingStrategy:
        unsafe extern "system" fn(*mut core::ffi::c_void, LineStackingStrategy) -> windows_core::HRESULT,
    pub IsTextSelectionEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsTextSelectionEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub SelectedText:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ContentStart:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ContentEnd:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SelectionStart:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SelectionEnd:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub BaselineOffset: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SelectionChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveSelectionChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ContextMenuOpening:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveContextMenuOpening: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub SelectAll: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Select: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Focus: unsafe extern "system" fn(*mut core::ffi::c_void, FocusState, *mut bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(ITextBox, ITextBox_Vtbl, 0xe48f5a8b_1dff_4352_a1f4_e516514ec882);
impl windows_core::RuntimeType for ITextBox {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl ITextBox {
    pub fn Text(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Text)(windows_core::Interface::as_raw(self), &mut result__).map(
                || {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                },
            )
        }
    }
    pub fn SetText(&self, value: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetText)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
            )
            .ok()
        }
    }
    pub fn SelectedText(&self) -> windows_core::Result<String> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SelectedText)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| {
                    let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                    hstring.to_string_lossy()
                })
        }
    }
    pub fn SetSelectedText(&self, value: &str) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetSelectedText)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
            )
            .ok()
        }
    }
    pub fn SelectionLength(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SelectionLength)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetSelectionLength(&self, value: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetSelectionLength)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn SelectionStart(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SelectionStart)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetSelectionStart(&self, value: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetSelectionStart)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn MaxLength(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MaxLength)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetMaxLength(&self, value: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMaxLength)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn IsReadOnly(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsReadOnly)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetIsReadOnly(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsReadOnly)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn AcceptsReturn(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AcceptsReturn)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetAcceptsReturn(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetAcceptsReturn)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn TextAlignment(&self) -> windows_core::Result<TextAlignment> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TextAlignment)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetTextAlignment(&self, value: TextAlignment) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTextAlignment)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn TextWrapping(&self) -> windows_core::Result<TextWrapping> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TextWrapping)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetTextWrapping(&self, value: TextWrapping) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTextWrapping)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn IsSpellCheckEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsSpellCheckEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIsSpellCheckEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsSpellCheckEnabled)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn IsTextPredictionEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsTextPredictionEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIsTextPredictionEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsTextPredictionEnabled)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn InputScope(&self) -> windows_core::Result<InputScope> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).InputScope)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetInputScope<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<InputScope>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetInputScope)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn TextChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<TextChangedEventArgs>) + 'static,
    {
        let handler: TextChangedEventHandler = {
            let com = windows_core::imp::DelegateBox::<TextChangedEventHandler, F>::new(
                &TextChangedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).TextChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveTextChanged,
            ))
        }
    }
    pub fn SelectionChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
    {
        let handler: RoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RoutedEventHandler, F>::new(
                &RoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).SelectionChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveSelectionChanged,
            ))
        }
    }
    pub fn ContextMenuOpening<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ContextMenuEventArgs>) + 'static,
    {
        let handler: ContextMenuOpeningEventHandler = {
            let com = windows_core::imp::DelegateBox::<ContextMenuOpeningEventHandler, F>::new(
                &ContextMenuOpeningEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ContextMenuOpening)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveContextMenuOpening,
            ))
        }
    }
    pub fn Select(&self, start: i32, length: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).Select)(windows_core::Interface::as_raw(self), start, length).ok()
        }
    }
    pub fn SelectAll(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SelectAll)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn GetRectFromCharacterIndex(&self, charindex: i32, trailingedge: bool) -> windows_core::Result<Rect> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetRectFromCharacterIndex)(
                windows_core::Interface::as_raw(self),
                charindex,
                trailingedge,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct ITextBox_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Text: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetText: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SelectedText:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetSelectedText:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SelectionLength: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetSelectionLength: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub SelectionStart: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetSelectionStart: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub MaxLength: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetMaxLength: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub IsReadOnly: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsReadOnly: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub AcceptsReturn: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetAcceptsReturn: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub TextAlignment: unsafe extern "system" fn(*mut core::ffi::c_void, *mut TextAlignment) -> windows_core::HRESULT,
    pub SetTextAlignment: unsafe extern "system" fn(*mut core::ffi::c_void, TextAlignment) -> windows_core::HRESULT,
    pub TextWrapping: unsafe extern "system" fn(*mut core::ffi::c_void, *mut TextWrapping) -> windows_core::HRESULT,
    pub SetTextWrapping: unsafe extern "system" fn(*mut core::ffi::c_void, TextWrapping) -> windows_core::HRESULT,
    pub IsSpellCheckEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsSpellCheckEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub IsTextPredictionEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsTextPredictionEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub InputScope:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetInputScope:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub TextChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveTextChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub SelectionChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveSelectionChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ContextMenuOpening:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveContextMenuOpening: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Select: unsafe extern "system" fn(*mut core::ffi::c_void, i32, i32) -> windows_core::HRESULT,
    pub SelectAll: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetRectFromCharacterIndex:
        unsafe extern "system" fn(*mut core::ffi::c_void, i32, bool, *mut Rect) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ITextChangedEventArgs,
    ITextChangedEventArgs_Vtbl,
    0x4dd04f7d_7a11_4b2e_9933_577df39252b6
);
impl windows_core::RuntimeType for ITextChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ITextChangedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(ITextElement, ITextElement_Vtbl, 0xe83b0062_d776_4f92_baea_40e77d4791d5);
impl windows_core::RuntimeType for ITextElement {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ITextElement_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(ITextPointer, ITextPointer_Vtbl, 0xac687aa1_6a41_43ff_851e_45348aa2cf7b);
impl windows_core::RuntimeType for ITextPointer {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ITextPointer_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ITimedMetadataTrack,
    ITimedMetadataTrack_Vtbl,
    0x9e6aed9e_f67a_49a9_b330_cf03b0e9cf07
);
impl windows_core::RuntimeType for ITimedMetadataTrack {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ITimedMetadataTrack_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IToggleMenuFlyoutItem,
    IToggleMenuFlyoutItem_Vtbl,
    0x49841edd_88e9_4663_a701_cd4fd25e398f
);
impl windows_core::RuntimeType for IToggleMenuFlyoutItem {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IToggleMenuFlyoutItem {
    pub fn IsChecked(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsChecked)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetIsChecked(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsChecked)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
}
#[repr(C)]
pub struct IToggleMenuFlyoutItem_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub IsChecked: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsChecked: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(ITransform, ITransform_Vtbl, 0x4df74078_bfd6_4ed1_9682_d2fd8bf2fe6f);
impl windows_core::RuntimeType for ITransform {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ITransform_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(ITransition, ITransition_Vtbl, 0x3c677c7c_01d0_4dce_b333_976f93312b08);
impl windows_core::RuntimeType for ITransition {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ITransition_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ITranslateTransform,
    ITranslateTransform_Vtbl,
    0xc975905c_3c36_4229_817b_178f64c0e113
);
impl windows_core::RuntimeType for ITranslateTransform {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl ITranslateTransform {
    pub fn X(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).X)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetX(&self, value: f64) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetX)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn Y(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Y)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetY(&self, value: f64) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetY)(windows_core::Interface::as_raw(self), value).ok() }
    }
}
#[repr(C)]
pub struct ITranslateTransform_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub X: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetX: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub Y: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetY: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(ITriggerBase, ITriggerBase_Vtbl, 0xe7ea222f_dee6_4393_a8b2_8923d641f395);
impl windows_core::RuntimeType for ITriggerBase {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ITriggerBase_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IUIElement, IUIElement_Vtbl, 0x676d0be9_b65c_41c6_ba40_58cf87f201c1);
impl windows_core::RuntimeType for IUIElement {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IUIElement {
    pub fn DesiredSize(&self) -> windows_core::Result<Size> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DesiredSize)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn AllowDrop(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AllowDrop)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetAllowDrop(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetAllowDrop)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn Opacity(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Opacity)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetOpacity(&self, value: f64) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).SetOpacity)(windows_core::Interface::as_raw(self), value).ok() }
    }
    pub fn Clip(&self) -> windows_core::Result<RectangleGeometry> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Clip)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetClip<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<RectangleGeometry>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetClip)(windows_core::Interface::as_raw(self), value.param().abi())
                .ok()
        }
    }
    pub fn RenderTransform(&self) -> windows_core::Result<Transform> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RenderTransform)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetRenderTransform<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Transform>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetRenderTransform)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn Projection(&self) -> windows_core::Result<Projection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Projection)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetProjection<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Projection>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetProjection)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn RenderTransformOrigin(&self) -> windows_core::Result<Point> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RenderTransformOrigin)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetRenderTransformOrigin(&self, value: Point) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetRenderTransformOrigin)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub fn IsHitTestVisible(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsHitTestVisible)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIsHitTestVisible(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsHitTestVisible)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn Visibility(&self) -> windows_core::Result<Visibility> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Visibility)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetVisibility(&self, value: Visibility) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetVisibility)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn RenderSize(&self) -> windows_core::Result<Size> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RenderSize)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn UseLayoutRounding(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).UseLayoutRounding)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetUseLayoutRounding(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetUseLayoutRounding)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn Transitions(&self) -> windows_core::Result<TransitionCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Transitions)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetTransitions<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<TransitionCollection>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetTransitions)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn CacheMode(&self) -> windows_core::Result<CacheMode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CacheMode)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetCacheMode<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CacheMode>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetCacheMode)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn IsTapEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsTapEnabled)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn SetIsTapEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsTapEnabled)(windows_core::Interface::as_raw(self), value).ok()
        }
    }
    pub fn IsDoubleTapEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsDoubleTapEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIsDoubleTapEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsDoubleTapEnabled)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn IsRightTapEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsRightTapEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIsRightTapEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsRightTapEnabled)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn IsHoldingEnabled(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsHoldingEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetIsHoldingEnabled(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsHoldingEnabled)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn ManipulationMode(&self) -> windows_core::Result<ManipulationModes> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ManipulationMode)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetManipulationMode(&self, value: ManipulationModes) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetManipulationMode)(windows_core::Interface::as_raw(self), value)
                .ok()
        }
    }
    pub fn PointerCaptures(&self) -> windows_core::Result<windows_collections::IVectorView<Pointer>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PointerCaptures)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn KeyUp<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<KeyRoutedEventArgs>) + 'static,
    {
        let handler: KeyEventHandler = {
            let com =
                windows_core::imp::DelegateBox::<KeyEventHandler, F>::new(&KeyEventHandlerBox::<F>::VTABLE, handler);
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).KeyUp)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveKeyUp,
            ))
        }
    }
    pub fn KeyDown<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<KeyRoutedEventArgs>) + 'static,
    {
        let handler: KeyEventHandler = {
            let com =
                windows_core::imp::DelegateBox::<KeyEventHandler, F>::new(&KeyEventHandlerBox::<F>::VTABLE, handler);
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).KeyDown)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveKeyDown,
            ))
        }
    }
    pub fn GotFocus<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
    {
        let handler: RoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RoutedEventHandler, F>::new(
                &RoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).GotFocus)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveGotFocus,
            ))
        }
    }
    pub fn LostFocus<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
    {
        let handler: RoutedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RoutedEventHandler, F>::new(
                &RoutedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).LostFocus)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveLostFocus,
            ))
        }
    }
    pub fn DragEnter<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DragEventArgs>) + 'static,
    {
        let handler: DragEventHandler = {
            let com =
                windows_core::imp::DelegateBox::<DragEventHandler, F>::new(&DragEventHandlerBox::<F>::VTABLE, handler);
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DragEnter)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDragEnter,
            ))
        }
    }
    pub fn DragLeave<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DragEventArgs>) + 'static,
    {
        let handler: DragEventHandler = {
            let com =
                windows_core::imp::DelegateBox::<DragEventHandler, F>::new(&DragEventHandlerBox::<F>::VTABLE, handler);
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DragLeave)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDragLeave,
            ))
        }
    }
    pub fn DragOver<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DragEventArgs>) + 'static,
    {
        let handler: DragEventHandler = {
            let com =
                windows_core::imp::DelegateBox::<DragEventHandler, F>::new(&DragEventHandlerBox::<F>::VTABLE, handler);
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DragOver)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDragOver,
            ))
        }
    }
    pub fn Drop<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DragEventArgs>) + 'static,
    {
        let handler: DragEventHandler = {
            let com =
                windows_core::imp::DelegateBox::<DragEventHandler, F>::new(&DragEventHandlerBox::<F>::VTABLE, handler);
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Drop)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(self.clone(), token__, windows_core::Interface::vtable(self).RemoveDrop))
        }
    }
    pub fn PointerPressed<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>) + 'static,
    {
        let handler: PointerEventHandler = {
            let com = windows_core::imp::DelegateBox::<PointerEventHandler, F>::new(
                &PointerEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PointerPressed)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePointerPressed,
            ))
        }
    }
    pub fn PointerMoved<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>) + 'static,
    {
        let handler: PointerEventHandler = {
            let com = windows_core::imp::DelegateBox::<PointerEventHandler, F>::new(
                &PointerEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PointerMoved)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePointerMoved,
            ))
        }
    }
    pub fn PointerReleased<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>) + 'static,
    {
        let handler: PointerEventHandler = {
            let com = windows_core::imp::DelegateBox::<PointerEventHandler, F>::new(
                &PointerEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PointerReleased)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePointerReleased,
            ))
        }
    }
    pub fn PointerEntered<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>) + 'static,
    {
        let handler: PointerEventHandler = {
            let com = windows_core::imp::DelegateBox::<PointerEventHandler, F>::new(
                &PointerEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PointerEntered)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePointerEntered,
            ))
        }
    }
    pub fn PointerExited<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>) + 'static,
    {
        let handler: PointerEventHandler = {
            let com = windows_core::imp::DelegateBox::<PointerEventHandler, F>::new(
                &PointerEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PointerExited)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePointerExited,
            ))
        }
    }
    pub fn PointerCaptureLost<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>) + 'static,
    {
        let handler: PointerEventHandler = {
            let com = windows_core::imp::DelegateBox::<PointerEventHandler, F>::new(
                &PointerEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PointerCaptureLost)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePointerCaptureLost,
            ))
        }
    }
    pub fn PointerCanceled<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>) + 'static,
    {
        let handler: PointerEventHandler = {
            let com = windows_core::imp::DelegateBox::<PointerEventHandler, F>::new(
                &PointerEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PointerCanceled)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePointerCanceled,
            ))
        }
    }
    pub fn PointerWheelChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>) + 'static,
    {
        let handler: PointerEventHandler = {
            let com = windows_core::imp::DelegateBox::<PointerEventHandler, F>::new(
                &PointerEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).PointerWheelChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemovePointerWheelChanged,
            ))
        }
    }
    pub fn Tapped<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<TappedRoutedEventArgs>) + 'static,
    {
        let handler: TappedEventHandler = {
            let com = windows_core::imp::DelegateBox::<TappedEventHandler, F>::new(
                &TappedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Tapped)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveTapped,
            ))
        }
    }
    pub fn DoubleTapped<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<DoubleTappedRoutedEventArgs>) + 'static,
    {
        let handler: DoubleTappedEventHandler = {
            let com = windows_core::imp::DelegateBox::<DoubleTappedEventHandler, F>::new(
                &DoubleTappedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).DoubleTapped)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveDoubleTapped,
            ))
        }
    }
    pub fn Holding<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<HoldingRoutedEventArgs>) + 'static,
    {
        let handler: HoldingEventHandler = {
            let com = windows_core::imp::DelegateBox::<HoldingEventHandler, F>::new(
                &HoldingEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Holding)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveHolding,
            ))
        }
    }
    pub fn RightTapped<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RightTappedRoutedEventArgs>) + 'static,
    {
        let handler: RightTappedEventHandler = {
            let com = windows_core::imp::DelegateBox::<RightTappedEventHandler, F>::new(
                &RightTappedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).RightTapped)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveRightTapped,
            ))
        }
    }
    pub fn ManipulationStarting<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationStartingRoutedEventArgs>)
            + 'static,
    {
        let handler: ManipulationStartingEventHandler = {
            let com = windows_core::imp::DelegateBox::<ManipulationStartingEventHandler, F>::new(
                &ManipulationStartingEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ManipulationStarting)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveManipulationStarting,
            ))
        }
    }
    pub fn ManipulationInertiaStarting<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(
                windows_core::Ref<windows_core::IInspectable>,
                windows_core::Ref<ManipulationInertiaStartingRoutedEventArgs>,
            ) + 'static,
    {
        let handler: ManipulationInertiaStartingEventHandler = {
            let com = windows_core::imp::DelegateBox::<ManipulationInertiaStartingEventHandler, F>::new(
                &ManipulationInertiaStartingEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ManipulationInertiaStarting)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveManipulationInertiaStarting,
            ))
        }
    }
    pub fn ManipulationStarted<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationStartedRoutedEventArgs>)
            + 'static,
    {
        let handler: ManipulationStartedEventHandler = {
            let com = windows_core::imp::DelegateBox::<ManipulationStartedEventHandler, F>::new(
                &ManipulationStartedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ManipulationStarted)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveManipulationStarted,
            ))
        }
    }
    pub fn ManipulationDelta<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationDeltaRoutedEventArgs>)
            + 'static,
    {
        let handler: ManipulationDeltaEventHandler = {
            let com = windows_core::imp::DelegateBox::<ManipulationDeltaEventHandler, F>::new(
                &ManipulationDeltaEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ManipulationDelta)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveManipulationDelta,
            ))
        }
    }
    pub fn ManipulationCompleted<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationCompletedRoutedEventArgs>)
            + 'static,
    {
        let handler: ManipulationCompletedEventHandler = {
            let com = windows_core::imp::DelegateBox::<ManipulationCompletedEventHandler, F>::new(
                &ManipulationCompletedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).ManipulationCompleted)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveManipulationCompleted,
            ))
        }
    }
    pub fn Measure(&self, availablesize: Size) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).Measure)(windows_core::Interface::as_raw(self), availablesize).ok()
        }
    }
    pub fn Arrange(&self, finalrect: Rect) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).Arrange)(windows_core::Interface::as_raw(self), finalrect).ok()
        }
    }
    pub fn CapturePointer<P0>(&self, value: P0) -> windows_core::Result<bool>
    where
        P0: windows_core::Param<Pointer>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CapturePointer)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ReleasePointerCapture<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Pointer>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ReleasePointerCapture)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn ReleasePointerCaptures(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).ReleasePointerCaptures)(windows_core::Interface::as_raw(self)).ok()
        }
    }
    pub fn AddHandler<P0, P1>(&self, routedevent: P0, handler: P1, handledeventstoo: bool) -> windows_core::Result<()>
    where
        P0: windows_core::Param<RoutedEvent>,
        P1: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddHandler)(
                windows_core::Interface::as_raw(self),
                routedevent.param().abi(),
                handler.param().abi(),
                handledeventstoo,
            )
            .ok()
        }
    }
    pub fn RemoveHandler<P0, P1>(&self, routedevent: P0, handler: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<RoutedEvent>,
        P1: windows_core::Param<windows_core::IInspectable>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveHandler)(
                windows_core::Interface::as_raw(self),
                routedevent.param().abi(),
                handler.param().abi(),
            )
            .ok()
        }
    }
    pub fn TransformToVisual<P0>(&self, visual: P0) -> windows_core::Result<GeneralTransform>
    where
        P0: windows_core::Param<UIElement>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TransformToVisual)(
                windows_core::Interface::as_raw(self),
                visual.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn InvalidateMeasure(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).InvalidateMeasure)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn InvalidateArrange(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).InvalidateArrange)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn UpdateLayout(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).UpdateLayout)(windows_core::Interface::as_raw(self)).ok() }
    }
}
#[repr(C)]
pub struct IUIElement_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub DesiredSize: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Size) -> windows_core::HRESULT,
    pub AllowDrop: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetAllowDrop: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub Opacity: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub SetOpacity: unsafe extern "system" fn(*mut core::ffi::c_void, f64) -> windows_core::HRESULT,
    pub Clip: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetClip: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub RenderTransform:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetRenderTransform:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Projection:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetProjection:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub RenderTransformOrigin: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Point) -> windows_core::HRESULT,
    pub SetRenderTransformOrigin: unsafe extern "system" fn(*mut core::ffi::c_void, Point) -> windows_core::HRESULT,
    pub IsHitTestVisible: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsHitTestVisible: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub Visibility: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Visibility) -> windows_core::HRESULT,
    pub SetVisibility: unsafe extern "system" fn(*mut core::ffi::c_void, Visibility) -> windows_core::HRESULT,
    pub RenderSize: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Size) -> windows_core::HRESULT,
    pub UseLayoutRounding: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetUseLayoutRounding: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub Transitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetTransitions:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CacheMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetCacheMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub IsTapEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsTapEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub IsDoubleTapEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsDoubleTapEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub IsRightTapEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsRightTapEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub IsHoldingEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsHoldingEnabled: unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub ManipulationMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut ManipulationModes) -> windows_core::HRESULT,
    pub SetManipulationMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, ManipulationModes) -> windows_core::HRESULT,
    pub PointerCaptures:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub KeyUp:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveKeyUp: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub KeyDown:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveKeyDown: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub GotFocus:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveGotFocus: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub LostFocus:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveLostFocus: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub DragEnter:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDragEnter: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub DragLeave:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDragLeave: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub DragOver:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDragOver: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Drop:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDrop: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PointerPressed:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePointerPressed: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PointerMoved:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePointerMoved: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PointerReleased:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePointerReleased: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PointerEntered:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePointerEntered: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PointerExited:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePointerExited: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PointerCaptureLost:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePointerCaptureLost: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PointerCanceled:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePointerCanceled: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub PointerWheelChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemovePointerWheelChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Tapped:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveTapped: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub DoubleTapped:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveDoubleTapped: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Holding:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveHolding: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub RightTapped:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveRightTapped: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ManipulationStarting:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveManipulationStarting: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ManipulationInertiaStarting:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveManipulationInertiaStarting:
        unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ManipulationStarted:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveManipulationStarted: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ManipulationDelta:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveManipulationDelta: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub ManipulationCompleted:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveManipulationCompleted: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Measure: unsafe extern "system" fn(*mut core::ffi::c_void, Size) -> windows_core::HRESULT,
    pub Arrange: unsafe extern "system" fn(*mut core::ffi::c_void, Rect) -> windows_core::HRESULT,
    pub CapturePointer:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub ReleasePointerCapture:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ReleasePointerCaptures: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub AddHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        bool,
    ) -> windows_core::HRESULT,
    pub RemoveHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub TransformToVisual: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub InvalidateMeasure: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub InvalidateArrange: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub UpdateLayout: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IUriRuntimeClass, IUriRuntimeClass_Vtbl, 0x9e365e57_48b2_4160_956f_c7385120bbfc);
impl windows_core::RuntimeType for IUriRuntimeClass {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IUriRuntimeClass_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IUriRuntimeClassFactory,
    IUriRuntimeClassFactory_Vtbl,
    0x44a9796f_723e_4fdf_a218_033e75b0c084
);
impl windows_core::RuntimeType for IUriRuntimeClassFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IUriRuntimeClassFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateUri: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateWithRelativeUri: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IVector<T>(windows_core::IUnknown, core::marker::PhantomData<T>)
where
    T: windows_core::RuntimeType + 'static;
impl<T: windows_core::RuntimeType + 'static> windows_core::imp::CanInto<windows_core::IUnknown> for IVector<T> {}
impl<T: windows_core::RuntimeType + 'static> windows_core::imp::CanInto<windows_core::IInspectable> for IVector<T> {}
unsafe impl<T: windows_core::RuntimeType + 'static> windows_core::Interface for IVector<T> {
    type Vtable = IVector_Vtbl<T>;
    const IID: windows_core::GUID = windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
}
impl<T: windows_core::RuntimeType + 'static> windows_core::RuntimeType for IVector<T> {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
        .push_slice(b"pinterface({913337e9-11a1-4345-a3a2-4e7f956e222d}")
        .push_slice(b";")
        .push_other(T::SIGNATURE)
        .push_slice(b")");
}
impl<T: windows_core::RuntimeType + 'static> IVector<T> {
    pub fn GetAt(&self, index: u32) -> windows_core::Result<T> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAt)(windows_core::Interface::as_raw(self), index, &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Size(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Size)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn GetView(&self) -> windows_core::Result<windows_collections::IVectorView<T>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetView)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn IndexOf<P0>(&self, value: P0, index: &mut u32) -> windows_core::Result<bool>
    where
        P0: windows_core::Param<T>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IndexOf)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
                index,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn SetAt<P1>(&self, index: u32, value: P1) -> windows_core::Result<()>
    where
        P1: windows_core::Param<T>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetAt)(
                windows_core::Interface::as_raw(self),
                index,
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn InsertAt<P1>(&self, index: u32, value: P1) -> windows_core::Result<()>
    where
        P1: windows_core::Param<T>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertAt)(
                windows_core::Interface::as_raw(self),
                index,
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn RemoveAt(&self, index: u32) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).RemoveAt)(windows_core::Interface::as_raw(self), index).ok() }
    }
    pub fn Append<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<T>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Append)(windows_core::Interface::as_raw(self), value.param().abi())
                .ok()
        }
    }
    pub fn RemoveAtEnd(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).RemoveAtEnd)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn Clear(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).Clear)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn GetMany(
        &self,
        startindex: u32,
        items: &mut [<T as windows_core::imp::Type<T>>::Default],
    ) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetMany)(
                windows_core::Interface::as_raw(self),
                startindex,
                items.len().try_into().unwrap(),
                core::mem::transmute_copy(&items),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ReplaceAll(&self, items: &[<T as windows_core::imp::Type<T>>::Default]) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).ReplaceAll)(
                windows_core::Interface::as_raw(self),
                items.len().try_into().unwrap(),
                core::mem::transmute(items.as_ptr()),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct IVector_Vtbl<T>
where
    T: windows_core::RuntimeType + 'static,
{
    pub base__: windows_core::IInspectable_Vtbl,
    pub GetAt: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut windows_core::imp::AbiType<T>,
    ) -> windows_core::HRESULT,
    pub Size: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub GetView:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub IndexOf: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::imp::AbiType<T>,
        *mut u32,
        *mut bool,
    ) -> windows_core::HRESULT,
    pub SetAt:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, windows_core::imp::AbiType<T>) -> windows_core::HRESULT,
    pub InsertAt:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, windows_core::imp::AbiType<T>) -> windows_core::HRESULT,
    pub RemoveAt: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub Append:
        unsafe extern "system" fn(*mut core::ffi::c_void, windows_core::imp::AbiType<T>) -> windows_core::HRESULT,
    pub RemoveAtEnd: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Clear: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetMany: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        u32,
        *mut windows_core::imp::AbiType<T>,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub ReplaceAll: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const windows_core::imp::AbiType<T>,
    ) -> windows_core::HRESULT,
    T: core::marker::PhantomData<T>,
}
windows_core::imp::define_interface!(
    IVisibilityChangedEventArgs,
    IVisibilityChangedEventArgs_Vtbl,
    0xbf9918ea_d801_4564_a495_b1e84f8ad085
);
impl windows_core::RuntimeType for IVisibilityChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IVisibilityChangedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IWindow, IWindow_Vtbl, 0x3276167d_c9f6_462d_9de2_ae4c1fd8c2e5);
impl windows_core::RuntimeType for IWindow {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IWindow {
    pub fn Bounds(&self) -> windows_core::Result<Rect> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Bounds)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn Visible(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Visible)(windows_core::Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
    pub fn Content(&self) -> windows_core::Result<UIElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Content)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn SetContent<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<UIElement>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetContent)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub fn CoreWindow(&self) -> windows_core::Result<CoreWindow> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CoreWindow)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Dispatcher)(windows_core::Interface::as_raw(self), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub fn Activated<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<WindowActivatedEventArgs>) + 'static,
    {
        let handler: WindowActivatedEventHandler = {
            let com = windows_core::imp::DelegateBox::<WindowActivatedEventHandler, F>::new(
                &WindowActivatedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Activated)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveActivated,
            ))
        }
    }
    pub fn Closed<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<CoreWindowEventArgs>) + 'static,
    {
        let handler: WindowClosedEventHandler = {
            let com = windows_core::imp::DelegateBox::<WindowClosedEventHandler, F>::new(
                &WindowClosedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Closed)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveClosed,
            ))
        }
    }
    pub fn SizeChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<WindowSizeChangedEventArgs>) + 'static,
    {
        let handler: WindowSizeChangedEventHandler = {
            let com = windows_core::imp::DelegateBox::<WindowSizeChangedEventHandler, F>::new(
                &WindowSizeChangedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).SizeChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveSizeChanged,
            ))
        }
    }
    pub fn VisibilityChanged<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<VisibilityChangedEventArgs>) + 'static,
    {
        let handler: WindowVisibilityChangedEventHandler = {
            let com = windows_core::imp::DelegateBox::<WindowVisibilityChangedEventHandler, F>::new(
                &WindowVisibilityChangedEventHandlerBox::<F>::VTABLE,
                handler,
            );
            unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
        };
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).VisibilityChanged)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveVisibilityChanged,
            ))
        }
    }
    pub fn Activate(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).Activate)(windows_core::Interface::as_raw(self)).ok() }
    }
    pub fn Close(&self) -> windows_core::Result<()> {
        unsafe { (windows_core::Interface::vtable(self).Close)(windows_core::Interface::as_raw(self)).ok() }
    }
}
#[repr(C)]
pub struct IWindow_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Bounds: unsafe extern "system" fn(*mut core::ffi::c_void, *mut Rect) -> windows_core::HRESULT,
    pub Visible: unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub Content:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetContent: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CoreWindow:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Dispatcher:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Activated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveActivated: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Closed:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveClosed: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub SizeChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveSizeChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub VisibilityChanged:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut i64) -> windows_core::HRESULT,
    pub RemoveVisibilityChanged: unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub Activate: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Close: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IWindowActivatedEventArgs,
    IWindowActivatedEventArgs_Vtbl,
    0x179d65e7_4658_4cb6_aa13_41d094ea255e
);
impl windows_core::RuntimeType for IWindowActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IWindowActivatedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IWindowCreatedEventArgs,
    IWindowCreatedEventArgs_Vtbl,
    0x31b71470_feff_4654_af48_9b398ab5772b
);
impl windows_core::RuntimeType for IWindowCreatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IWindowCreatedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IWindowSizeChangedEventArgs,
    IWindowSizeChangedEventArgs_Vtbl,
    0x5a200ec7_0426_47dc_b86c_6f475915e451
);
impl windows_core::RuntimeType for IWindowSizeChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IWindowSizeChangedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(IWindowStatics, IWindowStatics_Vtbl, 0x93328409_4ea1_4afa_83dc_0c4e73e88bb1);
impl windows_core::RuntimeType for IWindowStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IWindowStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Current:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(IXamlReader, IXamlReader_Vtbl, 0x24374cf1_cceb_48bf_a514_41b0186f84c2);
impl windows_core::RuntimeType for IXamlReader {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IXamlReader_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IXamlReaderStatics,
    IXamlReaderStatics_Vtbl,
    0x9891c6bd_534f_4955_b85a_8a8dc0dca602
);
impl windows_core::RuntimeType for IXamlReaderStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IXamlReaderStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Load: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub LoadWithInitialTemplateValidation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IdleDispatchedHandler,
    IdleDispatchedHandler_Vtbl,
    0xa42b0c24_7f21_4abc_99c1_8f01007f0880
);
impl windows_core::RuntimeType for IdleDispatchedHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IdleDispatchedHandler {
    pub fn new<F: Fn(windows_core::Ref<IdleDispatchedHandlerArgs>) + 'static>(invoke: F) -> Self {
        let com = windows_core::imp::DelegateBox::<Self, F>::new(&IdleDispatchedHandlerBox::<F>::VTABLE, invoke);
        unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
    }
}
#[repr(C)]
pub struct IdleDispatchedHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(this: *mut core::ffi::c_void, e: *mut core::ffi::c_void) -> windows_core::HRESULT,
}
struct IdleDispatchedHandlerBox<F: Fn(windows_core::Ref<IdleDispatchedHandlerArgs>) + 'static>(
    core::marker::PhantomData<(fn() -> F,)>,
);
impl<F: Fn(windows_core::Ref<IdleDispatchedHandlerArgs>) + 'static> IdleDispatchedHandlerBox<F> {
    const VTABLE: IdleDispatchedHandler_Vtbl = IdleDispatchedHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<IdleDispatchedHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<IdleDispatchedHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<IdleDispatchedHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(this: *mut core::ffi::c_void, e: *mut core::ffi::c_void) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<IdleDispatchedHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdleDispatchedHandlerArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(IdleDispatchedHandlerArgs, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for IdleDispatchedHandlerArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IIdleDispatchedHandlerArgs>();
}
unsafe impl windows_core::Interface for IdleDispatchedHandlerArgs {
    type Vtable = <IIdleDispatchedHandlerArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IIdleDispatchedHandlerArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for IdleDispatchedHandlerArgs {
    type Target = IIdleDispatchedHandlerArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for IdleDispatchedHandlerArgs {
    const NAME: &'static str = "Windows.UI.Core.IdleDispatchedHandlerArgs";
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Image(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Image, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Image, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for Image {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IImage>();
}
unsafe impl windows_core::Interface for Image {
    type Vtable = <IImage as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IImage as windows_core::Interface>::IID;
}
impl core::ops::Deref for Image {
    type Target = IImage;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Image {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.Image";
}
unsafe impl Send for Image {}
unsafe impl Sync for Image {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageSource(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ImageSource, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ImageSource, DependencyObject);
impl windows_core::RuntimeType for ImageSource {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IImageSource>();
}
unsafe impl windows_core::Interface for ImageSource {
    type Vtable = <IImageSource as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IImageSource as windows_core::Interface>::IID;
}
impl core::ops::Deref for ImageSource {
    type Target = IImageSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ImageSource {
    const NAME: &'static str = "Windows.UI.Xaml.Media.ImageSource";
}
unsafe impl Send for ImageSource {}
unsafe impl Sync for ImageSource {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InMemoryRandomAccessStream(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InMemoryRandomAccessStream,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IRandomAccessStream
);
windows_core::imp::required_hierarchy!(InMemoryRandomAccessStream, IClosable, IInputStream, IOutputStream);
impl InMemoryRandomAccessStream {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<R, F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<InMemoryRandomAccessStream, windows_core::imp::IGenericFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for InMemoryRandomAccessStream {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IRandomAccessStream>();
}
unsafe impl windows_core::Interface for InMemoryRandomAccessStream {
    type Vtable = <IRandomAccessStream as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IRandomAccessStream as windows_core::Interface>::IID;
}
impl core::ops::Deref for InMemoryRandomAccessStream {
    type Target = IRandomAccessStream;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for InMemoryRandomAccessStream {
    const NAME: &'static str = "Windows.Storage.Streams.InMemoryRandomAccessStream";
}
unsafe impl Send for InMemoryRandomAccessStream {}
unsafe impl Sync for InMemoryRandomAccessStream {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IncrementalLoadingTrigger(pub i32);
impl IncrementalLoadingTrigger {
    pub const None: Self = Self(0);
    pub const Edge: Self = Self(1);
}
impl windows_core::imp::TypeKind for IncrementalLoadingTrigger {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for IncrementalLoadingTrigger {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Controls.IncrementalLoadingTrigger;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inline(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Inline, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Inline, TextElement, DependencyObject);
impl windows_core::RuntimeType for Inline {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IInline>();
}
unsafe impl windows_core::Interface for Inline {
    type Vtable = <IInline as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IInline as windows_core::Interface>::IID;
}
impl core::ops::Deref for Inline {
    type Target = IInline;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Inline {
    const NAME: &'static str = "Windows.UI.Xaml.Documents.Inline";
}
unsafe impl Send for Inline {}
unsafe impl Sync for Inline {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InlineCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InlineCollection,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IVector<Inline>
);
impl windows_core::RuntimeType for InlineCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector<Inline>>();
}
unsafe impl windows_core::Interface for InlineCollection {
    type Vtable = <IVector<Inline> as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVector<Inline> as windows_core::Interface>::IID;
}
impl core::ops::Deref for InlineCollection {
    type Target = IVector<Inline>;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for InlineCollection {
    const NAME: &'static str = "Windows.UI.Xaml.Documents.InlineCollection";
}
unsafe impl Send for InlineCollection {}
unsafe impl Sync for InlineCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputScope(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(InputScope, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(InputScope, DependencyObject);
impl windows_core::RuntimeType for InputScope {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IInputScope>();
}
unsafe impl windows_core::Interface for InputScope {
    type Vtable = <IInputScope as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IInputScope as windows_core::Interface>::IID;
}
impl core::ops::Deref for InputScope {
    type Target = IInputScope;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for InputScope {
    const NAME: &'static str = "Windows.UI.Xaml.Input.InputScope";
}
unsafe impl Send for InputScope {}
unsafe impl Sync for InputScope {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InputStreamOptions(pub u32);
impl InputStreamOptions {
    pub const None: Self = Self(0);
    pub const Partial: Self = Self(1);
    pub const ReadAhead: Self = Self(2);
}
impl windows_core::imp::TypeKind for InputStreamOptions {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for InputStreamOptions {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Storage.Streams.InputStreamOptions;u4)");
}
impl InputStreamOptions {
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
impl core::ops::BitOr for InputStreamOptions {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl core::ops::BitAnd for InputStreamOptions {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
impl core::ops::BitOrAssign for InputStreamOptions {
    fn bitor_assign(&mut self, other: Self) {
        self.0.bitor_assign(other.0);
    }
}
impl core::ops::BitAndAssign for InputStreamOptions {
    fn bitand_assign(&mut self, other: Self) {
        self.0.bitand_assign(other.0);
    }
}
impl core::ops::Not for InputStreamOptions {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.not())
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemClickEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ItemClickEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ItemClickEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for ItemClickEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IItemClickEventArgs>();
}
unsafe impl windows_core::Interface for ItemClickEventArgs {
    type Vtable = <IItemClickEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IItemClickEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for ItemClickEventArgs {
    type Target = IItemClickEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ItemClickEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ItemClickEventArgs";
}
unsafe impl Send for ItemClickEventArgs {}
unsafe impl Sync for ItemClickEventArgs {}
windows_core::imp::define_interface!(
    ItemClickEventHandler,
    ItemClickEventHandler_Vtbl,
    0x3df6d14e_e18a_4a75_9395_627c5f3cd489
);
impl windows_core::RuntimeType for ItemClickEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ItemClickEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct ItemClickEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ItemClickEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ItemClickEventArgs>) + 'static>
    ItemClickEventHandlerBox<F>
{
    const VTABLE: ItemClickEventHandler_Vtbl = ItemClickEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<ItemClickEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<ItemClickEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<ItemClickEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<ItemClickEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ItemCollection,
    windows_core::IUnknown,
    windows_core::IInspectable,
    windows_collections::IObservableVector<windows_core::IInspectable>
);
windows_core::imp::required_hierarchy!(ItemCollection, IVector<windows_core::IInspectable>);
impl windows_core::RuntimeType for ItemCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<
        Self,
        windows_collections::IObservableVector<windows_core::IInspectable>,
    >();
}
unsafe impl windows_core::Interface for ItemCollection {
    type Vtable =
        <windows_collections::IObservableVector<windows_core::IInspectable> as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <windows_collections::IObservableVector<windows_core::IInspectable> as windows_core::Interface>::IID;
}
impl core::ops::Deref for ItemCollection {
    type Target = windows_collections::IObservableVector<windows_core::IInspectable>;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ItemCollection {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ItemCollection";
}
unsafe impl Send for ItemCollection {}
unsafe impl Sync for ItemCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemContainerGenerator(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ItemContainerGenerator, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for ItemContainerGenerator {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IItemContainerGenerator>();
}
unsafe impl windows_core::Interface for ItemContainerGenerator {
    type Vtable = <IItemContainerGenerator as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IItemContainerGenerator as windows_core::Interface>::IID;
}
impl core::ops::Deref for ItemContainerGenerator {
    type Target = IItemContainerGenerator;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ItemContainerGenerator {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ItemContainerGenerator";
}
unsafe impl Send for ItemContainerGenerator {}
unsafe impl Sync for ItemContainerGenerator {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemsControl(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ItemsControl, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ItemsControl, Control, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for ItemsControl {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IItemsControl>();
}
unsafe impl windows_core::Interface for ItemsControl {
    type Vtable = <IItemsControl as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IItemsControl as windows_core::Interface>::IID;
}
impl core::ops::Deref for ItemsControl {
    type Target = IItemsControl;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ItemsControl {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ItemsControl";
}
unsafe impl Send for ItemsControl {}
unsafe impl Sync for ItemsControl {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemsPanelTemplate(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ItemsPanelTemplate, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ItemsPanelTemplate, FrameworkTemplate, DependencyObject);
impl windows_core::RuntimeType for ItemsPanelTemplate {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IItemsPanelTemplate>();
}
unsafe impl windows_core::Interface for ItemsPanelTemplate {
    type Vtable = <IItemsPanelTemplate as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IItemsPanelTemplate as windows_core::Interface>::IID;
}
impl core::ops::Deref for ItemsPanelTemplate {
    type Target = IItemsPanelTemplate;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ItemsPanelTemplate {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ItemsPanelTemplate";
}
unsafe impl Send for ItemsPanelTemplate {}
unsafe impl Sync for ItemsPanelTemplate {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemsWrapGrid(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ItemsWrapGrid, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ItemsWrapGrid, Panel, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for ItemsWrapGrid {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IItemsWrapGrid>();
}
unsafe impl windows_core::Interface for ItemsWrapGrid {
    type Vtable = <IItemsWrapGrid as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IItemsWrapGrid as windows_core::Interface>::IID;
}
impl core::ops::Deref for ItemsWrapGrid {
    type Target = IItemsWrapGrid;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ItemsWrapGrid {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ItemsWrapGrid";
}
unsafe impl Send for ItemsWrapGrid {}
unsafe impl Sync for ItemsWrapGrid {}
windows_core::imp::define_interface!(KeyEventHandler, KeyEventHandler_Vtbl, 0x7c63d2e5_7a0e_4e12_b96a_7715aa6ff1c8);
impl windows_core::RuntimeType for KeyEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct KeyEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct KeyEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<KeyRoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<KeyRoutedEventArgs>) + 'static>
    KeyEventHandlerBox<F>
{
    const VTABLE: KeyEventHandler_Vtbl = KeyEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<KeyEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<KeyEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<KeyEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this =
                &mut *(this as *mut *mut core::ffi::c_void as *mut windows_core::imp::DelegateBox<KeyEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(KeyRoutedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(KeyRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for KeyRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IKeyRoutedEventArgs>();
}
unsafe impl windows_core::Interface for KeyRoutedEventArgs {
    type Vtable = <IKeyRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IKeyRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for KeyRoutedEventArgs {
    type Target = IKeyRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for KeyRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.KeyRoutedEventArgs";
}
unsafe impl Send for KeyRoutedEventArgs {}
unsafe impl Sync for KeyRoutedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KeyboardNavigationMode(pub i32);
impl KeyboardNavigationMode {
    pub const Local: Self = Self(0);
    pub const Cycle: Self = Self(1);
    pub const Once: Self = Self(2);
}
impl windows_core::imp::TypeKind for KeyboardNavigationMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for KeyboardNavigationMode {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Input.KeyboardNavigationMode;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchActivatedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    LaunchActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable,
    ILaunchActivatedEventArgs
);
windows_core::imp::required_hierarchy!(LaunchActivatedEventArgs, IActivatedEventArgs);
impl windows_core::RuntimeType for LaunchActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ILaunchActivatedEventArgs>();
}
unsafe impl windows_core::Interface for LaunchActivatedEventArgs {
    type Vtable = <ILaunchActivatedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ILaunchActivatedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for LaunchActivatedEventArgs {
    type Target = ILaunchActivatedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for LaunchActivatedEventArgs {
    const NAME: &'static str = "Windows.ApplicationModel.Activation.LaunchActivatedEventArgs";
}
unsafe impl Send for LaunchActivatedEventArgs {}
unsafe impl Sync for LaunchActivatedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LineStackingStrategy(pub i32);
impl LineStackingStrategy {
    pub const MaxHeight: Self = Self(0);
    pub const BlockLineHeight: Self = Self(1);
    pub const BaselineToBaseline: Self = Self(2);
}
impl windows_core::imp::TypeKind for LineStackingStrategy {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for LineStackingStrategy {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.LineStackingStrategy;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListView(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ListView, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(
    ListView,
    ListViewBase,
    Selector,
    ItemsControl,
    Control,
    FrameworkElement,
    UIElement,
    DependencyObject
);
impl windows_core::RuntimeType for ListView {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IListView>();
}
unsafe impl windows_core::Interface for ListView {
    type Vtable = <IListView as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IListView as windows_core::Interface>::IID;
}
impl core::ops::Deref for ListView {
    type Target = IListView;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ListView {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ListView";
}
unsafe impl Send for ListView {}
unsafe impl Sync for ListView {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListViewBase(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ListViewBase, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(
    ListViewBase,
    Selector,
    ItemsControl,
    Control,
    FrameworkElement,
    UIElement,
    DependencyObject
);
impl windows_core::RuntimeType for ListViewBase {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IListViewBase>();
}
unsafe impl windows_core::Interface for ListViewBase {
    type Vtable = <IListViewBase as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IListViewBase as windows_core::Interface>::IID;
}
impl core::ops::Deref for ListViewBase {
    type Target = IListViewBase;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ListViewBase {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ListViewBase";
}
unsafe impl Send for ListViewBase {}
unsafe impl Sync for ListViewBase {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListViewItem(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ListViewItem, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(
    ListViewItem,
    SelectorItem,
    ContentControl,
    Control,
    FrameworkElement,
    UIElement,
    DependencyObject
);
impl windows_core::RuntimeType for ListViewItem {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IListViewItem>();
}
unsafe impl windows_core::Interface for ListViewItem {
    type Vtable = <IListViewItem as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IListViewItem as windows_core::Interface>::IID;
}
impl core::ops::Deref for ListViewItem {
    type Target = IListViewItem;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ListViewItem {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ListViewItem";
}
unsafe impl Send for ListViewItem {}
unsafe impl Sync for ListViewItem {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ListViewSelectionMode(pub i32);
impl ListViewSelectionMode {
    pub const None: Self = Self(0);
    pub const Single: Self = Self(1);
    pub const Multiple: Self = Self(2);
    pub const Extended: Self = Self(3);
}
impl windows_core::imp::TypeKind for ListViewSelectionMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for ListViewSelectionMode {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Controls.ListViewSelectionMode;i4)");
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LoadMoreItemsResult {
    pub count: u32,
}
impl windows_core::imp::TypeKind for LoadMoreItemsResult {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for LoadMoreItemsResult {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.UI.Xaml.Data.LoadMoreItemsResult;u4)");
}
windows_core::imp::define_interface!(
    ManipulationCompletedEventHandler,
    ManipulationCompletedEventHandler_Vtbl,
    0x38ef4b0f_14f8_42df_9a1e_a4bcc4af77f4
);
impl windows_core::RuntimeType for ManipulationCompletedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ManipulationCompletedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct ManipulationCompletedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationCompletedRoutedEventArgs>)
        + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationCompletedRoutedEventArgs>)
        + 'static,
> ManipulationCompletedEventHandlerBox<F>
{
    const VTABLE: ManipulationCompletedEventHandler_Vtbl = ManipulationCompletedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<ManipulationCompletedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<ManipulationCompletedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<ManipulationCompletedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<ManipulationCompletedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManipulationCompletedRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ManipulationCompletedRoutedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(ManipulationCompletedRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for ManipulationCompletedRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IManipulationCompletedRoutedEventArgs>();
}
unsafe impl windows_core::Interface for ManipulationCompletedRoutedEventArgs {
    type Vtable = <IManipulationCompletedRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IManipulationCompletedRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for ManipulationCompletedRoutedEventArgs {
    type Target = IManipulationCompletedRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ManipulationCompletedRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.ManipulationCompletedRoutedEventArgs";
}
unsafe impl Send for ManipulationCompletedRoutedEventArgs {}
unsafe impl Sync for ManipulationCompletedRoutedEventArgs {}
windows_core::imp::define_interface!(
    ManipulationDeltaEventHandler,
    ManipulationDeltaEventHandler_Vtbl,
    0xaa1160cb_dfb9_4c56_abdc_711b63c8eb94
);
impl windows_core::RuntimeType for ManipulationDeltaEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ManipulationDeltaEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct ManipulationDeltaEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationDeltaRoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationDeltaRoutedEventArgs>) + 'static,
> ManipulationDeltaEventHandlerBox<F>
{
    const VTABLE: ManipulationDeltaEventHandler_Vtbl = ManipulationDeltaEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<ManipulationDeltaEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<ManipulationDeltaEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<ManipulationDeltaEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<ManipulationDeltaEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManipulationDeltaRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ManipulationDeltaRoutedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(ManipulationDeltaRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for ManipulationDeltaRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IManipulationDeltaRoutedEventArgs>();
}
unsafe impl windows_core::Interface for ManipulationDeltaRoutedEventArgs {
    type Vtable = <IManipulationDeltaRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IManipulationDeltaRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for ManipulationDeltaRoutedEventArgs {
    type Target = IManipulationDeltaRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ManipulationDeltaRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.ManipulationDeltaRoutedEventArgs";
}
unsafe impl Send for ManipulationDeltaRoutedEventArgs {}
unsafe impl Sync for ManipulationDeltaRoutedEventArgs {}
windows_core::imp::define_interface!(
    ManipulationInertiaStartingEventHandler,
    ManipulationInertiaStartingEventHandler_Vtbl,
    0xd39d6322_7c9c_481b_827b_c8b2d9bb6fc7
);
impl windows_core::RuntimeType for ManipulationInertiaStartingEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ManipulationInertiaStartingEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct ManipulationInertiaStartingEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationInertiaStartingRoutedEventArgs>)
        + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationInertiaStartingRoutedEventArgs>)
        + 'static,
> ManipulationInertiaStartingEventHandlerBox<F>
{
    const VTABLE: ManipulationInertiaStartingEventHandler_Vtbl = ManipulationInertiaStartingEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface:
                windows_core::imp::DelegateBox::<ManipulationInertiaStartingEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<ManipulationInertiaStartingEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<ManipulationInertiaStartingEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<ManipulationInertiaStartingEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManipulationInertiaStartingRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ManipulationInertiaStartingRoutedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(ManipulationInertiaStartingRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for ManipulationInertiaStartingRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IManipulationInertiaStartingRoutedEventArgs>();
}
unsafe impl windows_core::Interface for ManipulationInertiaStartingRoutedEventArgs {
    type Vtable = <IManipulationInertiaStartingRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IManipulationInertiaStartingRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for ManipulationInertiaStartingRoutedEventArgs {
    type Target = IManipulationInertiaStartingRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ManipulationInertiaStartingRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.ManipulationInertiaStartingRoutedEventArgs";
}
unsafe impl Send for ManipulationInertiaStartingRoutedEventArgs {}
unsafe impl Sync for ManipulationInertiaStartingRoutedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ManipulationModes(pub u32);
impl ManipulationModes {
    pub const None: Self = Self(0);
    pub const TranslateX: Self = Self(1);
    pub const TranslateY: Self = Self(2);
    pub const TranslateRailsX: Self = Self(4);
    pub const TranslateRailsY: Self = Self(8);
    pub const Rotate: Self = Self(16);
    pub const Scale: Self = Self(32);
    pub const TranslateInertia: Self = Self(64);
    pub const RotateInertia: Self = Self(128);
    pub const ScaleInertia: Self = Self(256);
    pub const All: Self = Self(65535);
    pub const System: Self = Self(65536);
}
impl windows_core::imp::TypeKind for ManipulationModes {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for ManipulationModes {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Input.ManipulationModes;u4)");
}
impl ManipulationModes {
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
impl core::ops::BitOr for ManipulationModes {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl core::ops::BitAnd for ManipulationModes {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
impl core::ops::BitOrAssign for ManipulationModes {
    fn bitor_assign(&mut self, other: Self) {
        self.0.bitor_assign(other.0);
    }
}
impl core::ops::BitAndAssign for ManipulationModes {
    fn bitand_assign(&mut self, other: Self) {
        self.0.bitand_assign(other.0);
    }
}
impl core::ops::Not for ManipulationModes {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.not())
    }
}
windows_core::imp::define_interface!(
    ManipulationStartedEventHandler,
    ManipulationStartedEventHandler_Vtbl,
    0xf88345f8_e0a3_4be2_b90c_dc20e6d8beb0
);
impl windows_core::RuntimeType for ManipulationStartedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ManipulationStartedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct ManipulationStartedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationStartedRoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationStartedRoutedEventArgs>) + 'static,
> ManipulationStartedEventHandlerBox<F>
{
    const VTABLE: ManipulationStartedEventHandler_Vtbl = ManipulationStartedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<ManipulationStartedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<ManipulationStartedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<ManipulationStartedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<ManipulationStartedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManipulationStartedRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ManipulationStartedRoutedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(ManipulationStartedRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for ManipulationStartedRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IManipulationStartedRoutedEventArgs>();
}
unsafe impl windows_core::Interface for ManipulationStartedRoutedEventArgs {
    type Vtable = <IManipulationStartedRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IManipulationStartedRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for ManipulationStartedRoutedEventArgs {
    type Target = IManipulationStartedRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ManipulationStartedRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.ManipulationStartedRoutedEventArgs";
}
unsafe impl Send for ManipulationStartedRoutedEventArgs {}
unsafe impl Sync for ManipulationStartedRoutedEventArgs {}
windows_core::imp::define_interface!(
    ManipulationStartingEventHandler,
    ManipulationStartingEventHandler_Vtbl,
    0x10d0b04e_bfe4_42cb_823c_3fecd8770ef8
);
impl windows_core::RuntimeType for ManipulationStartingEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ManipulationStartingEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct ManipulationStartingEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationStartingRoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<ManipulationStartingRoutedEventArgs>) + 'static,
> ManipulationStartingEventHandlerBox<F>
{
    const VTABLE: ManipulationStartingEventHandler_Vtbl = ManipulationStartingEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<ManipulationStartingEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<ManipulationStartingEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<ManipulationStartingEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<ManipulationStartingEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManipulationStartingRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ManipulationStartingRoutedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(ManipulationStartingRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for ManipulationStartingRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IManipulationStartingRoutedEventArgs>();
}
unsafe impl windows_core::Interface for ManipulationStartingRoutedEventArgs {
    type Vtable = <IManipulationStartingRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IManipulationStartingRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for ManipulationStartingRoutedEventArgs {
    type Target = IManipulationStartingRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ManipulationStartingRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.ManipulationStartingRoutedEventArgs";
}
unsafe impl Send for ManipulationStartingRoutedEventArgs {}
unsafe impl Sync for ManipulationStartingRoutedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaBinder(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaBinder, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for MediaBinder {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IMediaBinder>();
}
unsafe impl windows_core::Interface for MediaBinder {
    type Vtable = <IMediaBinder as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaBinder as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaBinder {
    type Target = IMediaBinder;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaBinder {
    const NAME: &'static str = "Windows.Media.Core.MediaBinder";
}
unsafe impl Send for MediaBinder {}
unsafe impl Sync for MediaBinder {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaBreakManager(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaBreakManager, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for MediaBreakManager {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaBreakManager>();
}
unsafe impl windows_core::Interface for MediaBreakManager {
    type Vtable = <IMediaBreakManager as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaBreakManager as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaBreakManager {
    type Target = IMediaBreakManager;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaBreakManager {
    const NAME: &'static str = "Windows.Media.Playback.MediaBreakManager";
}
unsafe impl Send for MediaBreakManager {}
unsafe impl Sync for MediaBreakManager {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaPlaybackAudioTrackList(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    MediaPlaybackAudioTrackList,
    windows_core::IUnknown,
    windows_core::IInspectable,
    windows_collections::IVectorView<AudioTrack>
);
impl windows_core::RuntimeType for MediaPlaybackAudioTrackList {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, windows_collections::IVectorView<AudioTrack>>();
}
unsafe impl windows_core::Interface for MediaPlaybackAudioTrackList {
    type Vtable = <windows_collections::IVectorView<AudioTrack> as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <windows_collections::IVectorView<AudioTrack> as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaPlaybackAudioTrackList {
    type Target = windows_collections::IVectorView<AudioTrack>;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaPlaybackAudioTrackList {
    const NAME: &'static str = "Windows.Media.Playback.MediaPlaybackAudioTrackList";
}
unsafe impl Send for MediaPlaybackAudioTrackList {}
unsafe impl Sync for MediaPlaybackAudioTrackList {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaPlaybackCommandManager(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    MediaPlaybackCommandManager,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for MediaPlaybackCommandManager {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaPlaybackCommandManager>();
}
unsafe impl windows_core::Interface for MediaPlaybackCommandManager {
    type Vtable = <IMediaPlaybackCommandManager as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaPlaybackCommandManager as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaPlaybackCommandManager {
    type Target = IMediaPlaybackCommandManager;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaPlaybackCommandManager {
    const NAME: &'static str = "Windows.Media.Playback.MediaPlaybackCommandManager";
}
unsafe impl Send for MediaPlaybackCommandManager {}
unsafe impl Sync for MediaPlaybackCommandManager {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaPlaybackItem(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaPlaybackItem, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(MediaPlaybackItem, IMediaPlaybackSource);
impl MediaPlaybackItem {
    pub fn Create<P0>(source: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<MediaSource>,
    {
        Self::IMediaPlaybackItemFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Create)(
                windows_core::Interface::as_raw(this),
                source.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IMediaPlaybackItemFactory<R, F: FnOnce(&IMediaPlaybackItemFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<MediaPlaybackItem, IMediaPlaybackItemFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for MediaPlaybackItem {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaPlaybackItem>();
}
unsafe impl windows_core::Interface for MediaPlaybackItem {
    type Vtable = <IMediaPlaybackItem as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaPlaybackItem as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaPlaybackItem {
    type Target = IMediaPlaybackItem;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaPlaybackItem {
    const NAME: &'static str = "Windows.Media.Playback.MediaPlaybackItem";
}
unsafe impl Send for MediaPlaybackItem {}
unsafe impl Sync for MediaPlaybackItem {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaPlaybackSession(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaPlaybackSession, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for MediaPlaybackSession {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaPlaybackSession>();
}
unsafe impl windows_core::Interface for MediaPlaybackSession {
    type Vtable = <IMediaPlaybackSession as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaPlaybackSession as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaPlaybackSession {
    type Target = IMediaPlaybackSession;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaPlaybackSession {
    const NAME: &'static str = "Windows.Media.Playback.MediaPlaybackSession";
}
unsafe impl Send for MediaPlaybackSession {}
unsafe impl Sync for MediaPlaybackSession {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MediaPlaybackState(pub i32);
impl MediaPlaybackState {
    pub const None: Self = Self(0);
    pub const Opening: Self = Self(1);
    pub const Buffering: Self = Self(2);
    pub const Playing: Self = Self(3);
    pub const Paused: Self = Self(4);
}
impl windows_core::imp::TypeKind for MediaPlaybackState {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for MediaPlaybackState {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Media.Playback.MediaPlaybackState;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaPlaybackTimedMetadataTrackList(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    MediaPlaybackTimedMetadataTrackList,
    windows_core::IUnknown,
    windows_core::IInspectable,
    windows_collections::IVectorView<TimedMetadataTrack>
);
impl windows_core::RuntimeType for MediaPlaybackTimedMetadataTrackList {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, windows_collections::IVectorView<TimedMetadataTrack>>();
}
unsafe impl windows_core::Interface for MediaPlaybackTimedMetadataTrackList {
    type Vtable = <windows_collections::IVectorView<TimedMetadataTrack> as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <windows_collections::IVectorView<TimedMetadataTrack> as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaPlaybackTimedMetadataTrackList {
    type Target = windows_collections::IVectorView<TimedMetadataTrack>;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaPlaybackTimedMetadataTrackList {
    const NAME: &'static str = "Windows.Media.Playback.MediaPlaybackTimedMetadataTrackList";
}
unsafe impl Send for MediaPlaybackTimedMetadataTrackList {}
unsafe impl Sync for MediaPlaybackTimedMetadataTrackList {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaPlaybackVideoTrackList(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    MediaPlaybackVideoTrackList,
    windows_core::IUnknown,
    windows_core::IInspectable,
    windows_collections::IVectorView<VideoTrack>
);
impl windows_core::RuntimeType for MediaPlaybackVideoTrackList {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, windows_collections::IVectorView<VideoTrack>>();
}
unsafe impl windows_core::Interface for MediaPlaybackVideoTrackList {
    type Vtable = <windows_collections::IVectorView<VideoTrack> as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <windows_collections::IVectorView<VideoTrack> as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaPlaybackVideoTrackList {
    type Target = windows_collections::IVectorView<VideoTrack>;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaPlaybackVideoTrackList {
    const NAME: &'static str = "Windows.Media.Playback.MediaPlaybackVideoTrackList";
}
unsafe impl Send for MediaPlaybackVideoTrackList {}
unsafe impl Sync for MediaPlaybackVideoTrackList {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaPlayer(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaPlayer, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(MediaPlayer, IClosable);
impl MediaPlayer {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<R, F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<MediaPlayer, windows_core::imp::IGenericFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for MediaPlayer {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IMediaPlayer>();
}
unsafe impl windows_core::Interface for MediaPlayer {
    type Vtable = <IMediaPlayer as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaPlayer as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaPlayer {
    type Target = IMediaPlayer;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaPlayer {
    const NAME: &'static str = "Windows.Media.Playback.MediaPlayer";
}
unsafe impl Send for MediaPlayer {}
unsafe impl Sync for MediaPlayer {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaPlayerElement(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaPlayerElement, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(MediaPlayerElement, Control, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for MediaPlayerElement {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaPlayerElement>();
}
unsafe impl windows_core::Interface for MediaPlayerElement {
    type Vtable = <IMediaPlayerElement as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaPlayerElement as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaPlayerElement {
    type Target = IMediaPlayerElement;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaPlayerElement {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.MediaPlayerElement";
}
unsafe impl Send for MediaPlayerElement {}
unsafe impl Sync for MediaPlayerElement {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MediaPlayerError(pub i32);
impl MediaPlayerError {
    pub const Unknown: Self = Self(0);
    pub const Aborted: Self = Self(1);
    pub const NetworkError: Self = Self(2);
    pub const DecodingError: Self = Self(3);
    pub const SourceNotSupported: Self = Self(4);
}
impl windows_core::imp::TypeKind for MediaPlayerError {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for MediaPlayerError {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Media.Playback.MediaPlayerError;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaPlayerFailedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaPlayerFailedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for MediaPlayerFailedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaPlayerFailedEventArgs>();
}
unsafe impl windows_core::Interface for MediaPlayerFailedEventArgs {
    type Vtable = <IMediaPlayerFailedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaPlayerFailedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaPlayerFailedEventArgs {
    type Target = IMediaPlayerFailedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaPlayerFailedEventArgs {
    const NAME: &'static str = "Windows.Media.Playback.MediaPlayerFailedEventArgs";
}
unsafe impl Send for MediaPlayerFailedEventArgs {}
unsafe impl Sync for MediaPlayerFailedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaPlayerRateChangedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    MediaPlayerRateChangedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for MediaPlayerRateChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaPlayerRateChangedEventArgs>();
}
unsafe impl windows_core::Interface for MediaPlayerRateChangedEventArgs {
    type Vtable = <IMediaPlayerRateChangedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaPlayerRateChangedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaPlayerRateChangedEventArgs {
    type Target = IMediaPlayerRateChangedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaPlayerRateChangedEventArgs {
    const NAME: &'static str = "Windows.Media.Playback.MediaPlayerRateChangedEventArgs";
}
unsafe impl Send for MediaPlayerRateChangedEventArgs {}
unsafe impl Sync for MediaPlayerRateChangedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MediaPlayerState(pub i32);
impl MediaPlayerState {
    pub const Closed: Self = Self(0);
    pub const Opening: Self = Self(1);
    pub const Buffering: Self = Self(2);
    pub const Playing: Self = Self(3);
    pub const Paused: Self = Self(4);
    pub const Stopped: Self = Self(5);
}
impl windows_core::imp::TypeKind for MediaPlayerState {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for MediaPlayerState {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Media.Playback.MediaPlayerState;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaSource(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaSource, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(MediaSource, IClosable, IMediaPlaybackSource);
impl MediaSource {
    pub fn CreateFromAdaptiveMediaSource<P0>(mediasource: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<AdaptiveMediaSource>,
    {
        Self::IMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromAdaptiveMediaSource)(
                windows_core::Interface::as_raw(this),
                mediasource.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromMediaStreamSource<P0>(mediasource: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<MediaStreamSource>,
    {
        Self::IMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromMediaStreamSource)(
                windows_core::Interface::as_raw(this),
                mediasource.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromMseStreamSource<P0>(mediasource: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<MseStreamSource>,
    {
        Self::IMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromMseStreamSource)(
                windows_core::Interface::as_raw(this),
                mediasource.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromIMediaSource<P0>(mediasource: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IMediaSource>,
    {
        Self::IMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromIMediaSource)(
                windows_core::Interface::as_raw(this),
                mediasource.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromStorageFile<P0>(file: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IStorageFile>,
    {
        Self::IMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromStorageFile)(
                windows_core::Interface::as_raw(this),
                file.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromStream<P0>(stream: P0, contenttype: &str) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IRandomAccessStream>,
    {
        Self::IMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromStream)(
                windows_core::Interface::as_raw(this),
                stream.param().abi(),
                core::mem::transmute_copy(&windows_core::HSTRING::from(contenttype)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromStreamReference<P0>(stream: P0, contenttype: &str) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IRandomAccessStreamReference>,
    {
        Self::IMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromStreamReference)(
                windows_core::Interface::as_raw(this),
                stream.param().abi(),
                core::mem::transmute_copy(&windows_core::HSTRING::from(contenttype)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromUri<P0>(uri: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<Uri>,
    {
        Self::IMediaSourceStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromUri)(
                windows_core::Interface::as_raw(this),
                uri.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateFromMediaBinder<P0>(binder: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<MediaBinder>,
    {
        Self::IMediaSourceStatics2(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromMediaBinder)(
                windows_core::Interface::as_raw(this),
                binder.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IMediaSourceStatics<R, F: FnOnce(&IMediaSourceStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<MediaSource, IMediaSourceStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    fn IMediaSourceStatics2<R, F: FnOnce(&IMediaSourceStatics2) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<MediaSource, IMediaSourceStatics2> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for MediaSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaSource2>();
}
unsafe impl windows_core::Interface for MediaSource {
    type Vtable = <IMediaSource2 as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaSource2 as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaSource {
    type Target = IMediaSource2;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaSource {
    const NAME: &'static str = "Windows.Media.Core.MediaSource";
}
unsafe impl Send for MediaSource {}
unsafe impl Sync for MediaSource {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaStreamSource(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaStreamSource, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for MediaStreamSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaStreamSource>();
}
unsafe impl windows_core::Interface for MediaStreamSource {
    type Vtable = <IMediaStreamSource as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaStreamSource as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaStreamSource {
    type Target = IMediaStreamSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaStreamSource {
    const NAME: &'static str = "Windows.Media.Core.MediaStreamSource";
}
unsafe impl Send for MediaStreamSource {}
unsafe impl Sync for MediaStreamSource {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaTimelineController(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaTimelineController, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for MediaTimelineController {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaTimelineController>();
}
unsafe impl windows_core::Interface for MediaTimelineController {
    type Vtable = <IMediaTimelineController as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaTimelineController as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaTimelineController {
    type Target = IMediaTimelineController;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaTimelineController {
    const NAME: &'static str = "Windows.Media.MediaTimelineController";
}
unsafe impl Send for MediaTimelineController {}
unsafe impl Sync for MediaTimelineController {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaTransportControls(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MediaTransportControls, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(MediaTransportControls, Control, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for MediaTransportControls {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMediaTransportControls>();
}
unsafe impl windows_core::Interface for MediaTransportControls {
    type Vtable = <IMediaTransportControls as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaTransportControls as windows_core::Interface>::IID;
}
impl core::ops::Deref for MediaTransportControls {
    type Target = IMediaTransportControls;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MediaTransportControls {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.MediaTransportControls";
}
unsafe impl Send for MediaTransportControls {}
unsafe impl Sync for MediaTransportControls {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuFlyout(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MenuFlyout, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(MenuFlyout, FlyoutBase, DependencyObject);
impl windows_core::RuntimeType for MenuFlyout {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IMenuFlyout>();
}
unsafe impl windows_core::Interface for MenuFlyout {
    type Vtable = <IMenuFlyout as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMenuFlyout as windows_core::Interface>::IID;
}
impl core::ops::Deref for MenuFlyout {
    type Target = IMenuFlyout;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MenuFlyout {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.MenuFlyout";
}
unsafe impl Send for MenuFlyout {}
unsafe impl Sync for MenuFlyout {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuFlyoutItem(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MenuFlyoutItem, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(
    MenuFlyoutItem,
    MenuFlyoutItemBase,
    Control,
    FrameworkElement,
    UIElement,
    DependencyObject
);
impl windows_core::RuntimeType for MenuFlyoutItem {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMenuFlyoutItem>();
}
unsafe impl windows_core::Interface for MenuFlyoutItem {
    type Vtable = <IMenuFlyoutItem as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMenuFlyoutItem as windows_core::Interface>::IID;
}
impl core::ops::Deref for MenuFlyoutItem {
    type Target = IMenuFlyoutItem;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MenuFlyoutItem {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.MenuFlyoutItem";
}
unsafe impl Send for MenuFlyoutItem {}
unsafe impl Sync for MenuFlyoutItem {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuFlyoutItemBase(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MenuFlyoutItemBase, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(MenuFlyoutItemBase, Control, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for MenuFlyoutItemBase {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMenuFlyoutItemBase>();
}
unsafe impl windows_core::Interface for MenuFlyoutItemBase {
    type Vtable = <IMenuFlyoutItemBase as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMenuFlyoutItemBase as windows_core::Interface>::IID;
}
impl core::ops::Deref for MenuFlyoutItemBase {
    type Target = IMenuFlyoutItemBase;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MenuFlyoutItemBase {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.MenuFlyoutItemBase";
}
unsafe impl Send for MenuFlyoutItemBase {}
unsafe impl Sync for MenuFlyoutItemBase {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MseStreamSource(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(MseStreamSource, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for MseStreamSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IMseStreamSource>();
}
unsafe impl windows_core::Interface for MseStreamSource {
    type Vtable = <IMseStreamSource as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMseStreamSource as windows_core::Interface>::IID;
}
impl core::ops::Deref for MseStreamSource {
    type Target = IMseStreamSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for MseStreamSource {
    const NAME: &'static str = "Windows.Media.Core.MseStreamSource";
}
unsafe impl Send for MseStreamSource {}
unsafe impl Sync for MseStreamSource {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NameCollisionOption(pub i32);
impl NameCollisionOption {
    pub const GenerateUniqueName: Self = Self(0);
    pub const ReplaceExisting: Self = Self(1);
    pub const FailIfExists: Self = Self(2);
}
impl windows_core::imp::TypeKind for NameCollisionOption {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for NameCollisionOption {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Storage.NameCollisionOption;i4)");
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Orientation(pub i32);
impl Orientation {
    pub const Vertical: Self = Self(0);
    pub const Horizontal: Self = Self(1);
}
impl windows_core::imp::TypeKind for Orientation {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Orientation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Controls.Orientation;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Panel(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Panel, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Panel, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for Panel {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IPanel>();
}
unsafe impl windows_core::Interface for Panel {
    type Vtable = <IPanel as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IPanel as windows_core::Interface>::IID;
}
impl core::ops::Deref for Panel {
    type Target = IPanel;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Panel {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.Panel";
}
unsafe impl Send for Panel {}
unsafe impl Sync for Panel {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PanelScrollingDirection(pub i32);
impl PanelScrollingDirection {
    pub const None: Self = Self(0);
    pub const Forward: Self = Self(1);
    pub const Backward: Self = Self(2);
}
impl windows_core::imp::TypeKind for PanelScrollingDirection {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for PanelScrollingDirection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Controls.PanelScrollingDirection;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PasswordCredential(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(PasswordCredential, windows_core::IUnknown, windows_core::IInspectable);
impl PasswordCredential {
    pub fn CreatePasswordCredential(resource: &str, username: &str, password: &str) -> windows_core::Result<Self> {
        Self::ICredentialFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreatePasswordCredential)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(&windows_core::HSTRING::from(resource)),
                core::mem::transmute_copy(&windows_core::HSTRING::from(username)),
                core::mem::transmute_copy(&windows_core::HSTRING::from(password)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn ICredentialFactory<R, F: FnOnce(&ICredentialFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<PasswordCredential, ICredentialFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for PasswordCredential {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IPasswordCredential>();
}
unsafe impl windows_core::Interface for PasswordCredential {
    type Vtable = <IPasswordCredential as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IPasswordCredential as windows_core::Interface>::IID;
}
impl core::ops::Deref for PasswordCredential {
    type Target = IPasswordCredential;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for PasswordCredential {
    const NAME: &'static str = "Windows.Security.Credentials.PasswordCredential";
}
unsafe impl Send for PasswordCredential {}
unsafe impl Sync for PasswordCredential {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PasswordVault(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(PasswordVault, windows_core::IUnknown, windows_core::IInspectable);
impl PasswordVault {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<R, F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<PasswordVault, windows_core::imp::IGenericFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for PasswordVault {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IPasswordVault>();
}
unsafe impl windows_core::Interface for PasswordVault {
    type Vtable = <IPasswordVault as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IPasswordVault as windows_core::Interface>::IID;
}
impl core::ops::Deref for PasswordVault {
    type Target = IPasswordVault;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for PasswordVault {
    const NAME: &'static str = "Windows.Security.Credentials.PasswordVault";
}
unsafe impl Send for PasswordVault {}
unsafe impl Sync for PasswordVault {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayToSource(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(PlayToSource, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for PlayToSource {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IPlayToSource>();
}
unsafe impl windows_core::Interface for PlayToSource {
    type Vtable = <IPlayToSource as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IPlayToSource as windows_core::Interface>::IID;
}
impl core::ops::Deref for PlayToSource {
    type Target = IPlayToSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for PlayToSource {
    const NAME: &'static str = "Windows.Media.PlayTo.PlayToSource";
}
unsafe impl Send for PlayToSource {}
unsafe impl Sync for PlayToSource {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaybackMediaMarkerReachedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    PlaybackMediaMarkerReachedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for PlaybackMediaMarkerReachedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IPlaybackMediaMarkerReachedEventArgs>();
}
unsafe impl windows_core::Interface for PlaybackMediaMarkerReachedEventArgs {
    type Vtable = <IPlaybackMediaMarkerReachedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IPlaybackMediaMarkerReachedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for PlaybackMediaMarkerReachedEventArgs {
    type Target = IPlaybackMediaMarkerReachedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for PlaybackMediaMarkerReachedEventArgs {
    const NAME: &'static str = "Windows.Media.Playback.PlaybackMediaMarkerReachedEventArgs";
}
unsafe impl Send for PlaybackMediaMarkerReachedEventArgs {}
unsafe impl Sync for PlaybackMediaMarkerReachedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaybackMediaMarkerSequence(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    PlaybackMediaMarkerSequence,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for PlaybackMediaMarkerSequence {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IPlaybackMediaMarkerSequence>();
}
unsafe impl windows_core::Interface for PlaybackMediaMarkerSequence {
    type Vtable = <IPlaybackMediaMarkerSequence as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IPlaybackMediaMarkerSequence as windows_core::Interface>::IID;
}
impl core::ops::Deref for PlaybackMediaMarkerSequence {
    type Target = IPlaybackMediaMarkerSequence;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for PlaybackMediaMarkerSequence {
    const NAME: &'static str = "Windows.Media.Playback.PlaybackMediaMarkerSequence";
}
unsafe impl Send for PlaybackMediaMarkerSequence {}
unsafe impl Sync for PlaybackMediaMarkerSequence {}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl windows_core::imp::TypeKind for Point {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Point {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.Foundation.Point;f4;f4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pointer(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Pointer, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for Pointer {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IPointer>();
}
unsafe impl windows_core::Interface for Pointer {
    type Vtable = <IPointer as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IPointer as windows_core::Interface>::IID;
}
impl core::ops::Deref for Pointer {
    type Target = IPointer;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Pointer {
    const NAME: &'static str = "Windows.UI.Xaml.Input.Pointer";
}
unsafe impl Send for Pointer {}
unsafe impl Sync for Pointer {}
windows_core::imp::define_interface!(
    PointerEventHandler,
    PointerEventHandler_Vtbl,
    0xe4385929_c004_4bcf_8970_359486e39f88
);
impl windows_core::RuntimeType for PointerEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct PointerEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct PointerEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<PointerRoutedEventArgs>) + 'static>
    PointerEventHandlerBox<F>
{
    const VTABLE: PointerEventHandler_Vtbl = PointerEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<PointerEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<PointerEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<PointerEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<PointerEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointerRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(PointerRoutedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(PointerRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for PointerRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IPointerRoutedEventArgs>();
}
unsafe impl windows_core::Interface for PointerRoutedEventArgs {
    type Vtable = <IPointerRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IPointerRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for PointerRoutedEventArgs {
    type Target = IPointerRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for PointerRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.PointerRoutedEventArgs";
}
unsafe impl Send for PointerRoutedEventArgs {}
unsafe impl Sync for PointerRoutedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Projection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Projection, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Projection, DependencyObject);
impl windows_core::RuntimeType for Projection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IProjection>();
}
unsafe impl windows_core::Interface for Projection {
    type Vtable = <IProjection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IProjection as windows_core::Interface>::IID;
}
impl core::ops::Deref for Projection {
    type Target = IProjection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Projection {
    const NAME: &'static str = "Windows.UI.Xaml.Media.Projection";
}
unsafe impl Send for Projection {}
unsafe impl Sync for Projection {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PropertyType(pub i32);
impl PropertyType {
    pub const Empty: Self = Self(0);
    pub const UInt8: Self = Self(1);
    pub const Int16: Self = Self(2);
    pub const UInt16: Self = Self(3);
    pub const Int32: Self = Self(4);
    pub const UInt32: Self = Self(5);
    pub const Int64: Self = Self(6);
    pub const UInt64: Self = Self(7);
    pub const Single: Self = Self(8);
    pub const Double: Self = Self(9);
    pub const Char16: Self = Self(10);
    pub const Boolean: Self = Self(11);
    pub const String: Self = Self(12);
    pub const Inspectable: Self = Self(13);
    pub const DateTime: Self = Self(14);
    pub const TimeSpan: Self = Self(15);
    pub const Guid: Self = Self(16);
    pub const Point: Self = Self(17);
    pub const Size: Self = Self(18);
    pub const Rect: Self = Self(19);
    pub const OtherType: Self = Self(20);
    pub const UInt8Array: Self = Self(1025);
    pub const Int16Array: Self = Self(1026);
    pub const UInt16Array: Self = Self(1027);
    pub const Int32Array: Self = Self(1028);
    pub const UInt32Array: Self = Self(1029);
    pub const Int64Array: Self = Self(1030);
    pub const UInt64Array: Self = Self(1031);
    pub const SingleArray: Self = Self(1032);
    pub const DoubleArray: Self = Self(1033);
    pub const Char16Array: Self = Self(1034);
    pub const BooleanArray: Self = Self(1035);
    pub const StringArray: Self = Self(1036);
    pub const InspectableArray: Self = Self(1037);
    pub const DateTimeArray: Self = Self(1038);
    pub const TimeSpanArray: Self = Self(1039);
    pub const GuidArray: Self = Self(1040);
    pub const PointArray: Self = Self(1041);
    pub const SizeArray: Self = Self(1042);
    pub const RectArray: Self = Self(1043);
    pub const OtherTypeArray: Self = Self(1044);
}
impl windows_core::imp::TypeKind for PropertyType {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for PropertyType {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Foundation.PropertyType;i4)");
}
pub struct PropertyValue;
impl PropertyValue {
    pub fn CreateEmpty() -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateEmpty)(windows_core::Interface::as_raw(this), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateUInt8(value: u8) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateUInt8)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateInt16(value: i16) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInt16)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateUInt16(value: u16) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateUInt16)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateInt32(value: i32) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInt32)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateUInt32(value: u32) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateUInt32)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateInt64(value: i64) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInt64)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateUInt64(value: u64) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateUInt64)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateSingle(value: f32) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateSingle)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateDouble(value: f64) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateDouble)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateChar16(value: u16) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateChar16)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateBoolean(value: bool) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateBoolean)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateString(value: &str) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateString)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(&windows_core::HSTRING::from(value)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateInspectable<P0>(value: P0) -> windows_core::Result<windows_core::IInspectable>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
    {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInspectable)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateGuid(value: windows_core::GUID) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateGuid)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateDateTime(value: windows_time::DateTime) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateDateTime)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateTimeSpan(value: windows_time::TimeSpan) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateTimeSpan)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreatePoint(value: Point) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreatePoint)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateSize(value: Size) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateSize)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateRect(value: Rect) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateRect)(
                windows_core::Interface::as_raw(this),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateUInt8Array(value: &[u8]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateUInt8Array)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateInt16Array(value: &[i16]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInt16Array)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateUInt16Array(value: &[u16]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateUInt16Array)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateInt32Array(value: &[i32]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInt32Array)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateUInt32Array(value: &[u32]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateUInt32Array)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateInt64Array(value: &[i64]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInt64Array)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateUInt64Array(value: &[u64]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateUInt64Array)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateSingleArray(value: &[f32]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateSingleArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateDoubleArray(value: &[f64]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateDoubleArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateChar16Array(value: &[u16]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateChar16Array)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateBooleanArray(value: &[bool]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateBooleanArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateStringArray(value: &[windows_core::HSTRING]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateStringArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                core::mem::transmute(value.as_ptr()),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateInspectableArray(
        value: &[Option<windows_core::IInspectable>],
    ) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInspectableArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                core::mem::transmute(value.as_ptr()),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateGuidArray(value: &[windows_core::GUID]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateGuidArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateDateTimeArray(value: &[windows_time::DateTime]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateDateTimeArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateTimeSpanArray(value: &[windows_time::TimeSpan]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateTimeSpanArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreatePointArray(value: &[Point]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreatePointArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateSizeArray(value: &[Size]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateSizeArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateRectArray(value: &[Rect]) -> windows_core::Result<windows_core::IInspectable> {
        Self::IPropertyValueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateRectArray)(
                windows_core::Interface::as_raw(this),
                value.len().try_into().unwrap(),
                value.as_ptr(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IPropertyValueStatics<R, F: FnOnce(&IPropertyValueStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<PropertyValue, IPropertyValueStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeName for PropertyValue {
    const NAME: &'static str = "Windows.Foundation.PropertyValue";
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl windows_core::imp::TypeKind for Rect {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Rect {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.Foundation.Rect;f4;f4;f4;f4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RectangleGeometry(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(RectangleGeometry, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(RectangleGeometry, Geometry, DependencyObject);
impl RectangleGeometry {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<R, F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<RectangleGeometry, windows_core::imp::IGenericFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for RectangleGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IRectangleGeometry>();
}
unsafe impl windows_core::Interface for RectangleGeometry {
    type Vtable = <IRectangleGeometry as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IRectangleGeometry as windows_core::Interface>::IID;
}
impl core::ops::Deref for RectangleGeometry {
    type Target = IRectangleGeometry;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for RectangleGeometry {
    const NAME: &'static str = "Windows.UI.Xaml.Media.RectangleGeometry";
}
unsafe impl Send for RectangleGeometry {}
unsafe impl Sync for RectangleGeometry {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceDictionary(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ResourceDictionary, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(ResourceDictionary, DependencyObject);
impl windows_core::RuntimeType for ResourceDictionary {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IResourceDictionary>();
}
unsafe impl windows_core::Interface for ResourceDictionary {
    type Vtable = <IResourceDictionary as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IResourceDictionary as windows_core::Interface>::IID;
}
impl core::ops::Deref for ResourceDictionary {
    type Target = IResourceDictionary;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ResourceDictionary {
    const NAME: &'static str = "Windows.UI.Xaml.ResourceDictionary";
}
unsafe impl Send for ResourceDictionary {}
unsafe impl Sync for ResourceDictionary {}
windows_core::imp::define_interface!(
    RightTappedEventHandler,
    RightTappedEventHandler_Vtbl,
    0x2532a062_f447_4950_9c46_f1e34a2c2238
);
impl windows_core::RuntimeType for RightTappedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct RightTappedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct RightTappedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RightTappedRoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RightTappedRoutedEventArgs>) + 'static>
    RightTappedEventHandlerBox<F>
{
    const VTABLE: RightTappedEventHandler_Vtbl = RightTappedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<RightTappedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<RightTappedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<RightTappedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<RightTappedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RightTappedRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(RightTappedRoutedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(RightTappedRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for RightTappedRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IRightTappedRoutedEventArgs>();
}
unsafe impl windows_core::Interface for RightTappedRoutedEventArgs {
    type Vtable = <IRightTappedRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IRightTappedRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for RightTappedRoutedEventArgs {
    type Target = IRightTappedRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for RightTappedRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.RightTappedRoutedEventArgs";
}
unsafe impl Send for RightTappedRoutedEventArgs {}
unsafe impl Sync for RightTappedRoutedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutedEvent(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(RoutedEvent, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for RoutedEvent {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IRoutedEvent>();
}
unsafe impl windows_core::Interface for RoutedEvent {
    type Vtable = <IRoutedEvent as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IRoutedEvent as windows_core::Interface>::IID;
}
impl core::ops::Deref for RoutedEvent {
    type Target = IRoutedEvent;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for RoutedEvent {
    const NAME: &'static str = "Windows.UI.Xaml.RoutedEvent";
}
unsafe impl Send for RoutedEvent {}
unsafe impl Sync for RoutedEvent {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(RoutedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for RoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IRoutedEventArgs>();
}
unsafe impl windows_core::Interface for RoutedEventArgs {
    type Vtable = <IRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for RoutedEventArgs {
    type Target = IRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for RoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.RoutedEventArgs";
}
unsafe impl Send for RoutedEventArgs {}
unsafe impl Sync for RoutedEventArgs {}
windows_core::imp::define_interface!(
    RoutedEventHandler,
    RoutedEventHandler_Vtbl,
    0xa856e674_b0b6_4bc3_bba8_1ba06e40d4b5
);
impl windows_core::RuntimeType for RoutedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct RoutedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct RoutedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<RoutedEventArgs>) + 'static>
    RoutedEventHandlerBox<F>
{
    const VTABLE: RoutedEventHandler_Vtbl = RoutedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<RoutedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<RoutedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<RoutedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<RoutedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowDefinition(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(RowDefinition, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(RowDefinition, DependencyObject);
impl windows_core::RuntimeType for RowDefinition {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IRowDefinition>();
}
unsafe impl windows_core::Interface for RowDefinition {
    type Vtable = <IRowDefinition as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IRowDefinition as windows_core::Interface>::IID;
}
impl core::ops::Deref for RowDefinition {
    type Target = IRowDefinition;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for RowDefinition {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.RowDefinition";
}
unsafe impl Send for RowDefinition {}
unsafe impl Sync for RowDefinition {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowDefinitionCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    RowDefinitionCollection,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IVector<RowDefinition>
);
impl windows_core::RuntimeType for RowDefinitionCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector<RowDefinition>>();
}
unsafe impl windows_core::Interface for RowDefinitionCollection {
    type Vtable = <IVector<RowDefinition> as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVector<RowDefinition> as windows_core::Interface>::IID;
}
impl core::ops::Deref for RowDefinitionCollection {
    type Target = IVector<RowDefinition>;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for RowDefinitionCollection {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.RowDefinitionCollection";
}
unsafe impl Send for RowDefinitionCollection {}
unsafe impl Sync for RowDefinitionCollection {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScrollIntoViewAlignment(pub i32);
impl ScrollIntoViewAlignment {
    pub const Default: Self = Self(0);
    pub const Leading: Self = Self(1);
}
impl windows_core::imp::TypeKind for ScrollIntoViewAlignment {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for ScrollIntoViewAlignment {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Controls.ScrollIntoViewAlignment;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchActivatedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SearchActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable,
    ISearchActivatedEventArgs
);
impl windows_core::RuntimeType for SearchActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISearchActivatedEventArgs>();
}
unsafe impl windows_core::Interface for SearchActivatedEventArgs {
    type Vtable = <ISearchActivatedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ISearchActivatedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for SearchActivatedEventArgs {
    type Target = ISearchActivatedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for SearchActivatedEventArgs {
    const NAME: &'static str = "Windows.ApplicationModel.Activation.SearchActivatedEventArgs";
}
unsafe impl Send for SearchActivatedEventArgs {}
unsafe impl Sync for SearchActivatedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selector(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Selector, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Selector, ItemsControl, Control, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for Selector {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ISelector>();
}
unsafe impl windows_core::Interface for Selector {
    type Vtable = <ISelector as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ISelector as windows_core::Interface>::IID;
}
impl core::ops::Deref for Selector {
    type Target = ISelector;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Selector {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.Primitives.Selector";
}
unsafe impl Send for Selector {}
unsafe impl Sync for Selector {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorItem(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(SelectorItem, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(
    SelectorItem,
    ContentControl,
    Control,
    FrameworkElement,
    UIElement,
    DependencyObject
);
impl windows_core::RuntimeType for SelectorItem {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISelectorItem>();
}
unsafe impl windows_core::Interface for SelectorItem {
    type Vtable = <ISelectorItem as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ISelectorItem as windows_core::Interface>::IID;
}
impl core::ops::Deref for SelectorItem {
    type Target = ISelectorItem;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for SelectorItem {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.Primitives.SelectorItem";
}
unsafe impl Send for SelectorItem {}
unsafe impl Sync for SelectorItem {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetVersionRequest(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(SetVersionRequest, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for SetVersionRequest {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISetVersionRequest>();
}
unsafe impl windows_core::Interface for SetVersionRequest {
    type Vtable = <ISetVersionRequest as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ISetVersionRequest as windows_core::Interface>::IID;
}
impl core::ops::Deref for SetVersionRequest {
    type Target = ISetVersionRequest;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for SetVersionRequest {
    const NAME: &'static str = "Windows.Storage.SetVersionRequest";
}
unsafe impl Send for SetVersionRequest {}
unsafe impl Sync for SetVersionRequest {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShareTargetActivatedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ShareTargetActivatedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IShareTargetActivatedEventArgs
);
impl windows_core::RuntimeType for ShareTargetActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IShareTargetActivatedEventArgs>();
}
unsafe impl windows_core::Interface for ShareTargetActivatedEventArgs {
    type Vtable = <IShareTargetActivatedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IShareTargetActivatedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for ShareTargetActivatedEventArgs {
    type Target = IShareTargetActivatedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ShareTargetActivatedEventArgs {
    const NAME: &'static str = "Windows.ApplicationModel.Activation.ShareTargetActivatedEventArgs";
}
unsafe impl Send for ShareTargetActivatedEventArgs {}
unsafe impl Sync for ShareTargetActivatedEventArgs {}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}
impl windows_core::imp::TypeKind for Size {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Size {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.Foundation.Size;f4;f4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SizeChangedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(SizeChangedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(SizeChangedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for SizeChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISizeChangedEventArgs>();
}
unsafe impl windows_core::Interface for SizeChangedEventArgs {
    type Vtable = <ISizeChangedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ISizeChangedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for SizeChangedEventArgs {
    type Target = ISizeChangedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for SizeChangedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.SizeChangedEventArgs";
}
unsafe impl Send for SizeChangedEventArgs {}
unsafe impl Sync for SizeChangedEventArgs {}
windows_core::imp::define_interface!(
    SizeChangedEventHandler,
    SizeChangedEventHandler_Vtbl,
    0x1115b13c_25d2_480b_89dc_eb3dcbd6b7fa
);
impl windows_core::RuntimeType for SizeChangedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct SizeChangedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct SizeChangedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<SizeChangedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<SizeChangedEventArgs>) + 'static>
    SizeChangedEventHandlerBox<F>
{
    const VTABLE: SizeChangedEventHandler_Vtbl = SizeChangedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<SizeChangedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<SizeChangedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<SizeChangedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<SizeChangedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SolidColorBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(SolidColorBrush, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(SolidColorBrush, Brush, DependencyObject);
impl SolidColorBrush {
    pub fn CreateInstanceWithColor(color: Color) -> windows_core::Result<Self> {
        Self::ISolidColorBrushFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInstanceWithColor)(
                windows_core::Interface::as_raw(this),
                color,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn ISolidColorBrushFactory<R, F: FnOnce(&ISolidColorBrushFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<SolidColorBrush, ISolidColorBrushFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for SolidColorBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISolidColorBrush>();
}
unsafe impl windows_core::Interface for SolidColorBrush {
    type Vtable = <ISolidColorBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ISolidColorBrush as windows_core::Interface>::IID;
}
impl core::ops::Deref for SolidColorBrush {
    type Target = ISolidColorBrush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for SolidColorBrush {
    const NAME: &'static str = "Windows.UI.Xaml.Media.SolidColorBrush";
}
unsafe impl Send for SolidColorBrush {}
unsafe impl Sync for SolidColorBrush {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StereoscopicVideoPackingMode(pub i32);
impl StereoscopicVideoPackingMode {
    pub const None: Self = Self(0);
    pub const SideBySide: Self = Self(1);
    pub const TopBottom: Self = Self(2);
}
impl windows_core::imp::TypeKind for StereoscopicVideoPackingMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for StereoscopicVideoPackingMode {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.Media.MediaProperties.StereoscopicVideoPackingMode;i4)",
    );
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StereoscopicVideoRenderMode(pub i32);
impl StereoscopicVideoRenderMode {
    pub const Mono: Self = Self(0);
    pub const Stereo: Self = Self(1);
}
impl windows_core::imp::TypeKind for StereoscopicVideoRenderMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for StereoscopicVideoRenderMode {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Media.Playback.StereoscopicVideoRenderMode;i4)");
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StorageDeleteOption(pub i32);
impl StorageDeleteOption {
    pub const Default: Self = Self(0);
    pub const PermanentDelete: Self = Self(1);
}
impl windows_core::imp::TypeKind for StorageDeleteOption {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for StorageDeleteOption {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Storage.StorageDeleteOption;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageFile(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(StorageFile, windows_core::IUnknown, windows_core::IInspectable, IStorageFile);
impl windows_core::RuntimeType for StorageFile {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IStorageFile>();
}
unsafe impl windows_core::Interface for StorageFile {
    type Vtable = <IStorageFile as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IStorageFile as windows_core::Interface>::IID;
}
impl core::ops::Deref for StorageFile {
    type Target = IStorageFile;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for StorageFile {
    const NAME: &'static str = "Windows.Storage.StorageFile";
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageFolder(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    StorageFolder,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IStorageFolder
);
windows_core::imp::required_hierarchy!(StorageFolder, IStorageItem);
impl windows_core::RuntimeType for StorageFolder {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IStorageFolder>();
}
unsafe impl windows_core::Interface for StorageFolder {
    type Vtable = <IStorageFolder as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IStorageFolder as windows_core::Interface>::IID;
}
impl core::ops::Deref for StorageFolder {
    type Target = IStorageFolder;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for StorageFolder {
    const NAME: &'static str = "Windows.Storage.StorageFolder";
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StorageItemTypes(pub u32);
impl StorageItemTypes {
    pub const None: Self = Self(0);
    pub const File: Self = Self(1);
    pub const Folder: Self = Self(2);
}
impl windows_core::imp::TypeKind for StorageItemTypes {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for StorageItemTypes {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Storage.StorageItemTypes;u4)");
}
impl StorageItemTypes {
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
impl core::ops::BitOr for StorageItemTypes {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl core::ops::BitAnd for StorageItemTypes {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
impl core::ops::BitOrAssign for StorageItemTypes {
    fn bitor_assign(&mut self, other: Self) {
        self.0.bitor_assign(other.0);
    }
}
impl core::ops::BitAndAssign for StorageItemTypes {
    fn bitand_assign(&mut self, other: Self) {
        self.0.bitand_assign(other.0);
    }
}
impl core::ops::Not for StorageItemTypes {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.not())
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Stretch(pub i32);
impl Stretch {
    pub const None: Self = Self(0);
    pub const Fill: Self = Self(1);
    pub const Uniform: Self = Self(2);
    pub const UniformToFill: Self = Self(3);
}
impl windows_core::imp::TypeKind for Stretch {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Stretch {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Media.Stretch;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Style(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Style, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Style, DependencyObject);
impl windows_core::RuntimeType for Style {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IStyle>();
}
unsafe impl windows_core::Interface for Style {
    type Vtable = <IStyle as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IStyle as windows_core::Interface>::IID;
}
impl core::ops::Deref for Style {
    type Target = IStyle;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Style {
    const NAME: &'static str = "Windows.UI.Xaml.Style";
}
unsafe impl Send for Style {}
unsafe impl Sync for Style {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleSelector(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(StyleSelector, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for StyleSelector {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IStyleSelector>();
}
unsafe impl windows_core::Interface for StyleSelector {
    type Vtable = <IStyleSelector as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IStyleSelector as windows_core::Interface>::IID;
}
impl core::ops::Deref for StyleSelector {
    type Target = IStyleSelector;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for StyleSelector {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.StyleSelector";
}
unsafe impl Send for StyleSelector {}
unsafe impl Sync for StyleSelector {}
windows_core::imp::define_interface!(
    TappedEventHandler,
    TappedEventHandler_Vtbl,
    0x68d940cc_9ff0_49ce_b141_3f07ec477b97
);
impl windows_core::RuntimeType for TappedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct TappedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct TappedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<TappedRoutedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<TappedRoutedEventArgs>) + 'static>
    TappedEventHandlerBox<F>
{
    const VTABLE: TappedEventHandler_Vtbl = TappedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<TappedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<TappedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<TappedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<TappedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TappedRoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(TappedRoutedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(TappedRoutedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for TappedRoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ITappedRoutedEventArgs>();
}
unsafe impl windows_core::Interface for TappedRoutedEventArgs {
    type Vtable = <ITappedRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITappedRoutedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for TappedRoutedEventArgs {
    type Target = ITappedRoutedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TappedRoutedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Input.TappedRoutedEventArgs";
}
unsafe impl Send for TappedRoutedEventArgs {}
unsafe impl Sync for TappedRoutedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextAlignment(pub i32);
impl TextAlignment {
    pub const Center: Self = Self(0);
    pub const Left: Self = Self(1);
    pub const Start: Self = Self(1);
    pub const Right: Self = Self(2);
    pub const End: Self = Self(2);
    pub const Justify: Self = Self(3);
    pub const DetectFromContent: Self = Self(4);
}
impl windows_core::imp::TypeKind for TextAlignment {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for TextAlignment {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.TextAlignment;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextBlock(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(TextBlock, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(TextBlock, FrameworkElement, UIElement, DependencyObject);
impl TextBlock {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<R, F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<TextBlock, windows_core::imp::IGenericFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for TextBlock {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ITextBlock>();
}
unsafe impl windows_core::Interface for TextBlock {
    type Vtable = <ITextBlock as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITextBlock as windows_core::Interface>::IID;
}
impl core::ops::Deref for TextBlock {
    type Target = ITextBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TextBlock {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.TextBlock";
}
unsafe impl Send for TextBlock {}
unsafe impl Sync for TextBlock {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextBox(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(TextBox, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(TextBox, Control, FrameworkElement, UIElement, DependencyObject);
impl windows_core::RuntimeType for TextBox {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ITextBox>();
}
unsafe impl windows_core::Interface for TextBox {
    type Vtable = <ITextBox as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITextBox as windows_core::Interface>::IID;
}
impl core::ops::Deref for TextBox {
    type Target = ITextBox;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TextBox {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.TextBox";
}
unsafe impl Send for TextBox {}
unsafe impl Sync for TextBox {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextChangedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(TextChangedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(TextChangedEventArgs, RoutedEventArgs);
impl windows_core::RuntimeType for TextChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ITextChangedEventArgs>();
}
unsafe impl windows_core::Interface for TextChangedEventArgs {
    type Vtable = <ITextChangedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITextChangedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for TextChangedEventArgs {
    type Target = ITextChangedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TextChangedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.TextChangedEventArgs";
}
unsafe impl Send for TextChangedEventArgs {}
unsafe impl Sync for TextChangedEventArgs {}
windows_core::imp::define_interface!(
    TextChangedEventHandler,
    TextChangedEventHandler_Vtbl,
    0x8eb35b97_ad87_40e8_818b_77db24759566
);
impl windows_core::RuntimeType for TextChangedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct TextChangedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct TextChangedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<TextChangedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<TextChangedEventArgs>) + 'static>
    TextChangedEventHandlerBox<F>
{
    const VTABLE: TextChangedEventHandler_Vtbl = TextChangedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<TextChangedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<TextChangedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<TextChangedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<TextChangedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextElement(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(TextElement, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(TextElement, DependencyObject);
impl windows_core::RuntimeType for TextElement {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ITextElement>();
}
unsafe impl windows_core::Interface for TextElement {
    type Vtable = <ITextElement as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITextElement as windows_core::Interface>::IID;
}
impl core::ops::Deref for TextElement {
    type Target = ITextElement;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TextElement {
    const NAME: &'static str = "Windows.UI.Xaml.Documents.TextElement";
}
unsafe impl Send for TextElement {}
unsafe impl Sync for TextElement {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextPointer(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(TextPointer, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for TextPointer {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ITextPointer>();
}
unsafe impl windows_core::Interface for TextPointer {
    type Vtable = <ITextPointer as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITextPointer as windows_core::Interface>::IID;
}
impl core::ops::Deref for TextPointer {
    type Target = ITextPointer;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TextPointer {
    const NAME: &'static str = "Windows.UI.Xaml.Documents.TextPointer";
}
unsafe impl Send for TextPointer {}
unsafe impl Sync for TextPointer {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextTrimming(pub i32);
impl TextTrimming {
    pub const None: Self = Self(0);
    pub const CharacterEllipsis: Self = Self(1);
    pub const WordEllipsis: Self = Self(2);
    pub const Clip: Self = Self(3);
}
impl windows_core::imp::TypeKind for TextTrimming {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for TextTrimming {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.TextTrimming;i4)");
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextWrapping(pub i32);
impl TextWrapping {
    pub const NoWrap: Self = Self(1);
    pub const Wrap: Self = Self(2);
    pub const WrapWholeWords: Self = Self(3);
}
impl windows_core::imp::TypeKind for TextWrapping {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for TextWrapping {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.TextWrapping;i4)");
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Thickness {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}
impl windows_core::imp::TypeKind for Thickness {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Thickness {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.UI.Xaml.Thickness;f8;f8;f8;f8)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimedMetadataTrack(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(TimedMetadataTrack, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for TimedMetadataTrack {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ITimedMetadataTrack>();
}
unsafe impl windows_core::Interface for TimedMetadataTrack {
    type Vtable = <ITimedMetadataTrack as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITimedMetadataTrack as windows_core::Interface>::IID;
}
impl core::ops::Deref for TimedMetadataTrack {
    type Target = ITimedMetadataTrack;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TimedMetadataTrack {
    const NAME: &'static str = "Windows.Media.Core.TimedMetadataTrack";
}
unsafe impl Send for TimedMetadataTrack {}
unsafe impl Sync for TimedMetadataTrack {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToggleMenuFlyoutItem(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(ToggleMenuFlyoutItem, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(
    ToggleMenuFlyoutItem,
    MenuFlyoutItem,
    MenuFlyoutItemBase,
    Control,
    FrameworkElement,
    UIElement,
    DependencyObject
);
impl windows_core::RuntimeType for ToggleMenuFlyoutItem {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IToggleMenuFlyoutItem>();
}
unsafe impl windows_core::Interface for ToggleMenuFlyoutItem {
    type Vtable = <IToggleMenuFlyoutItem as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IToggleMenuFlyoutItem as windows_core::Interface>::IID;
}
impl core::ops::Deref for ToggleMenuFlyoutItem {
    type Target = IToggleMenuFlyoutItem;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for ToggleMenuFlyoutItem {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.ToggleMenuFlyoutItem";
}
unsafe impl Send for ToggleMenuFlyoutItem {}
unsafe impl Sync for ToggleMenuFlyoutItem {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transform(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Transform, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Transform, GeneralTransform, DependencyObject);
impl windows_core::RuntimeType for Transform {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ITransform>();
}
unsafe impl windows_core::Interface for Transform {
    type Vtable = <ITransform as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITransform as windows_core::Interface>::IID;
}
impl core::ops::Deref for Transform {
    type Target = ITransform;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Transform {
    const NAME: &'static str = "Windows.UI.Xaml.Media.Transform";
}
unsafe impl Send for Transform {}
unsafe impl Sync for Transform {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transition(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Transition, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Transition, DependencyObject);
impl windows_core::RuntimeType for Transition {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ITransition>();
}
unsafe impl windows_core::Interface for Transition {
    type Vtable = <ITransition as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITransition as windows_core::Interface>::IID;
}
impl core::ops::Deref for Transition {
    type Target = ITransition;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Transition {
    const NAME: &'static str = "Windows.UI.Xaml.Media.Animation.Transition";
}
unsafe impl Send for Transition {}
unsafe impl Sync for Transition {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    TransitionCollection,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IVector<Transition>
);
impl windows_core::RuntimeType for TransitionCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector<Transition>>();
}
unsafe impl windows_core::Interface for TransitionCollection {
    type Vtable = <IVector<Transition> as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVector<Transition> as windows_core::Interface>::IID;
}
impl core::ops::Deref for TransitionCollection {
    type Target = IVector<Transition>;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TransitionCollection {
    const NAME: &'static str = "Windows.UI.Xaml.Media.Animation.TransitionCollection";
}
unsafe impl Send for TransitionCollection {}
unsafe impl Sync for TransitionCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslateTransform(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(TranslateTransform, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(TranslateTransform, Transform, GeneralTransform, DependencyObject);
impl TranslateTransform {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<R, F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<TranslateTransform, windows_core::imp::IGenericFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for TranslateTransform {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ITranslateTransform>();
}
unsafe impl windows_core::Interface for TranslateTransform {
    type Vtable = <ITranslateTransform as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITranslateTransform as windows_core::Interface>::IID;
}
impl core::ops::Deref for TranslateTransform {
    type Target = ITranslateTransform;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TranslateTransform {
    const NAME: &'static str = "Windows.UI.Xaml.Media.TranslateTransform";
}
unsafe impl Send for TranslateTransform {}
unsafe impl Sync for TranslateTransform {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerBase(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(TriggerBase, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(TriggerBase, DependencyObject);
impl windows_core::RuntimeType for TriggerBase {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, ITriggerBase>();
}
unsafe impl windows_core::Interface for TriggerBase {
    type Vtable = <ITriggerBase as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ITriggerBase as windows_core::Interface>::IID;
}
impl core::ops::Deref for TriggerBase {
    type Target = ITriggerBase;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TriggerBase {
    const NAME: &'static str = "Windows.UI.Xaml.TriggerBase";
}
unsafe impl Send for TriggerBase {}
unsafe impl Sync for TriggerBase {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    TriggerCollection,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IVector<TriggerBase>
);
impl windows_core::RuntimeType for TriggerCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector<TriggerBase>>();
}
unsafe impl windows_core::Interface for TriggerCollection {
    type Vtable = <IVector<TriggerBase> as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVector<TriggerBase> as windows_core::Interface>::IID;
}
impl core::ops::Deref for TriggerCollection {
    type Target = IVector<TriggerBase>;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for TriggerCollection {
    const NAME: &'static str = "Windows.UI.Xaml.TriggerCollection";
}
unsafe impl Send for TriggerCollection {}
unsafe impl Sync for TriggerCollection {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedEventHandler<TSender, TResult>(
    windows_core::IUnknown,
    core::marker::PhantomData<TSender>,
    core::marker::PhantomData<TResult>,
)
where
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static;
unsafe impl<TSender: windows_core::RuntimeType + 'static, TResult: windows_core::RuntimeType + 'static>
    windows_core::Interface for TypedEventHandler<TSender, TResult>
{
    type Vtable = TypedEventHandler_Vtbl<TSender, TResult>;
    const IID: windows_core::GUID = windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
}
impl<TSender: windows_core::RuntimeType + 'static, TResult: windows_core::RuntimeType + 'static>
    windows_core::RuntimeType for TypedEventHandler<TSender, TResult>
{
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
        .push_slice(b"pinterface({9de1c534-6ae1-11e0-84e1-18a905bcc53f}")
        .push_slice(b";")
        .push_other(TSender::SIGNATURE)
        .push_slice(b";")
        .push_other(TResult::SIGNATURE)
        .push_slice(b")");
}
impl<TSender: windows_core::RuntimeType + 'static, TResult: windows_core::RuntimeType + 'static>
    TypedEventHandler<TSender, TResult>
{
    pub fn new<F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) + 'static>(invoke: F) -> Self {
        let com = windows_core::imp::DelegateBox::<Self, F>::new(
            &TypedEventHandlerBox::<TSender, TResult, F>::VTABLE,
            invoke,
        );
        unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
    }
}
#[repr(C)]
pub struct TypedEventHandler_Vtbl<TSender, TResult>
where
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static,
{
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: windows_core::imp::AbiType<TSender>,
        args: windows_core::imp::AbiType<TResult>,
    ) -> windows_core::HRESULT,
    TSender: core::marker::PhantomData<TSender>,
    TResult: core::marker::PhantomData<TResult>,
}
struct TypedEventHandlerBox<TSender, TResult, F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) + 'static>(
    core::marker::PhantomData<(TSender, TResult, fn() -> F)>,
)
where
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static;
impl<
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static,
    F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) + 'static,
> TypedEventHandlerBox<TSender, TResult, F>
{
    const VTABLE: TypedEventHandler_Vtbl<TSender, TResult> = TypedEventHandler_Vtbl::<TSender, TResult> {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<TypedEventHandler<TSender, TResult>, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<TypedEventHandler<TSender, TResult>, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<TypedEventHandler<TSender, TResult>, F>::Release,
        },
        Invoke: Self::Invoke,
        TSender: core::marker::PhantomData::<TSender>,
        TResult: core::marker::PhantomData::<TResult>,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: windows_core::imp::AbiType<TSender>,
        args: windows_core::imp::AbiType<TResult>,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<TypedEventHandler<TSender, TResult>, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&args));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UIElement(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(UIElement, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(UIElement, DependencyObject);
impl windows_core::RuntimeType for UIElement {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IUIElement>();
}
unsafe impl windows_core::Interface for UIElement {
    type Vtable = <IUIElement as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IUIElement as windows_core::Interface>::IID;
}
impl core::ops::Deref for UIElement {
    type Target = IUIElement;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for UIElement {
    const NAME: &'static str = "Windows.UI.Xaml.UIElement";
}
unsafe impl Send for UIElement {}
unsafe impl Sync for UIElement {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UIElementCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    UIElementCollection,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IVector<UIElement>
);
impl windows_core::RuntimeType for UIElementCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector<UIElement>>();
}
unsafe impl windows_core::Interface for UIElementCollection {
    type Vtable = <IVector<UIElement> as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVector<UIElement> as windows_core::Interface>::IID;
}
impl core::ops::Deref for UIElementCollection {
    type Target = IVector<UIElement>;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for UIElementCollection {
    const NAME: &'static str = "Windows.UI.Xaml.Controls.UIElementCollection";
}
unsafe impl Send for UIElementCollection {}
unsafe impl Sync for UIElementCollection {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UnicodeEncoding(pub i32);
impl UnicodeEncoding {
    pub const Utf8: Self = Self(0);
    pub const Utf16LE: Self = Self(1);
    pub const Utf16BE: Self = Self(2);
}
impl windows_core::imp::TypeKind for UnicodeEncoding {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for UnicodeEncoding {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.Storage.Streams.UnicodeEncoding;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Uri(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Uri, windows_core::IUnknown, windows_core::IInspectable);
impl Uri {
    pub fn CreateUri(uri: &str) -> windows_core::Result<Self> {
        Self::IUriRuntimeClassFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateUri)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(&windows_core::HSTRING::from(uri)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn CreateWithRelativeUri(baseuri: &str, relativeuri: &str) -> windows_core::Result<Self> {
        Self::IUriRuntimeClassFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateWithRelativeUri)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(&windows_core::HSTRING::from(baseuri)),
                core::mem::transmute_copy(&windows_core::HSTRING::from(relativeuri)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IUriRuntimeClassFactory<R, F: FnOnce(&IUriRuntimeClassFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<Uri, IUriRuntimeClassFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for Uri {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IUriRuntimeClass>();
}
unsafe impl windows_core::Interface for Uri {
    type Vtable = <IUriRuntimeClass as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IUriRuntimeClass as windows_core::Interface>::IID;
}
impl core::ops::Deref for Uri {
    type Target = IUriRuntimeClass;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Uri {
    const NAME: &'static str = "Windows.Foundation.Uri";
}
unsafe impl Send for Uri {}
unsafe impl Sync for Uri {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VerticalAlignment(pub i32);
impl VerticalAlignment {
    pub const Top: Self = Self(0);
    pub const Center: Self = Self(1);
    pub const Bottom: Self = Self(2);
    pub const Stretch: Self = Self(3);
}
impl windows_core::imp::TypeKind for VerticalAlignment {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for VerticalAlignment {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.VerticalAlignment;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoTrack(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(VideoTrack, windows_core::IUnknown, windows_core::IInspectable, IMediaTrack);
impl windows_core::RuntimeType for VideoTrack {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IMediaTrack>();
}
unsafe impl windows_core::Interface for VideoTrack {
    type Vtable = <IMediaTrack as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IMediaTrack as windows_core::Interface>::IID;
}
impl core::ops::Deref for VideoTrack {
    type Target = IMediaTrack;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for VideoTrack {
    const NAME: &'static str = "Windows.Media.Core.VideoTrack";
}
unsafe impl Send for VideoTrack {}
unsafe impl Sync for VideoTrack {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Visibility(pub i32);
impl Visibility {
    pub const Visible: Self = Self(0);
    pub const Collapsed: Self = Self(1);
}
impl windows_core::imp::TypeKind for Visibility {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Visibility {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Windows.UI.Xaml.Visibility;i4)");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisibilityChangedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(VisibilityChangedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for VisibilityChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVisibilityChangedEventArgs>();
}
unsafe impl windows_core::Interface for VisibilityChangedEventArgs {
    type Vtable = <IVisibilityChangedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVisibilityChangedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for VisibilityChangedEventArgs {
    type Target = IVisibilityChangedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for VisibilityChangedEventArgs {
    const NAME: &'static str = "Windows.UI.Core.VisibilityChangedEventArgs";
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Window(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Window, windows_core::IUnknown, windows_core::IInspectable);
impl Window {
    pub fn Current() -> windows_core::Result<Self> {
        Self::IWindowStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Current)(windows_core::Interface::as_raw(this), &mut result__)
                .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IWindowStatics<R, F: FnOnce(&IWindowStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<Window, IWindowStatics> = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for Window {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IWindow>();
}
unsafe impl windows_core::Interface for Window {
    type Vtable = <IWindow as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IWindow as windows_core::Interface>::IID;
}
impl core::ops::Deref for Window {
    type Target = IWindow;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for Window {
    const NAME: &'static str = "Windows.UI.Xaml.Window";
}
unsafe impl Send for Window {}
unsafe impl Sync for Window {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowActivatedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(WindowActivatedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for WindowActivatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IWindowActivatedEventArgs>();
}
unsafe impl windows_core::Interface for WindowActivatedEventArgs {
    type Vtable = <IWindowActivatedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IWindowActivatedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for WindowActivatedEventArgs {
    type Target = IWindowActivatedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for WindowActivatedEventArgs {
    const NAME: &'static str = "Windows.UI.Core.WindowActivatedEventArgs";
}
windows_core::imp::define_interface!(
    WindowActivatedEventHandler,
    WindowActivatedEventHandler_Vtbl,
    0x18026348_8619_4c7b_b534_ced45d9de219
);
impl windows_core::RuntimeType for WindowActivatedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct WindowActivatedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct WindowActivatedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<WindowActivatedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<WindowActivatedEventArgs>) + 'static>
    WindowActivatedEventHandlerBox<F>
{
    const VTABLE: WindowActivatedEventHandler_Vtbl = WindowActivatedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<WindowActivatedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<WindowActivatedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<WindowActivatedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<WindowActivatedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
windows_core::imp::define_interface!(
    WindowClosedEventHandler,
    WindowClosedEventHandler_Vtbl,
    0x0db89161_20d7_45df_9122_ba89576703ba
);
impl windows_core::RuntimeType for WindowClosedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct WindowClosedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct WindowClosedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<CoreWindowEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<CoreWindowEventArgs>) + 'static>
    WindowClosedEventHandlerBox<F>
{
    const VTABLE: WindowClosedEventHandler_Vtbl = WindowClosedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<WindowClosedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<WindowClosedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<WindowClosedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<WindowClosedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowCreatedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(WindowCreatedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for WindowCreatedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IWindowCreatedEventArgs>();
}
unsafe impl windows_core::Interface for WindowCreatedEventArgs {
    type Vtable = <IWindowCreatedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IWindowCreatedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for WindowCreatedEventArgs {
    type Target = IWindowCreatedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for WindowCreatedEventArgs {
    const NAME: &'static str = "Windows.UI.Xaml.WindowCreatedEventArgs";
}
unsafe impl Send for WindowCreatedEventArgs {}
unsafe impl Sync for WindowCreatedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowSizeChangedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(WindowSizeChangedEventArgs, windows_core::IUnknown, windows_core::IInspectable);
impl windows_core::RuntimeType for WindowSizeChangedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IWindowSizeChangedEventArgs>();
}
unsafe impl windows_core::Interface for WindowSizeChangedEventArgs {
    type Vtable = <IWindowSizeChangedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IWindowSizeChangedEventArgs as windows_core::Interface>::IID;
}
impl core::ops::Deref for WindowSizeChangedEventArgs {
    type Target = IWindowSizeChangedEventArgs;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for WindowSizeChangedEventArgs {
    const NAME: &'static str = "Windows.UI.Core.WindowSizeChangedEventArgs";
}
windows_core::imp::define_interface!(
    WindowSizeChangedEventHandler,
    WindowSizeChangedEventHandler_Vtbl,
    0x5c21c742_2ced_4fd9_ba38_7118d40e966b
);
impl windows_core::RuntimeType for WindowSizeChangedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct WindowSizeChangedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct WindowSizeChangedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<WindowSizeChangedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<WindowSizeChangedEventArgs>) + 'static>
    WindowSizeChangedEventHandlerBox<F>
{
    const VTABLE: WindowSizeChangedEventHandler_Vtbl = WindowSizeChangedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<WindowSizeChangedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<WindowSizeChangedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<WindowSizeChangedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<WindowSizeChangedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
windows_core::imp::define_interface!(
    WindowVisibilityChangedEventHandler,
    WindowVisibilityChangedEventHandler_Vtbl,
    0x10406ad6_b090_4a4a_b2ad_d682df27130f
);
impl windows_core::RuntimeType for WindowVisibilityChangedEventHandler {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct WindowVisibilityChangedEventHandler_Vtbl {
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
struct WindowVisibilityChangedEventHandlerBox<
    F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<VisibilityChangedEventArgs>) + 'static,
>(core::marker::PhantomData<(fn() -> F,)>);
impl<F: Fn(windows_core::Ref<windows_core::IInspectable>, windows_core::Ref<VisibilityChangedEventArgs>) + 'static>
    WindowVisibilityChangedEventHandlerBox<F>
{
    const VTABLE: WindowVisibilityChangedEventHandler_Vtbl = WindowVisibilityChangedEventHandler_Vtbl {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<WindowVisibilityChangedEventHandler, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<WindowVisibilityChangedEventHandler, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<WindowVisibilityChangedEventHandler, F>::Release,
        },
        Invoke: Self::Invoke,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        e: *mut core::ffi::c_void,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<WindowVisibilityChangedEventHandler, F>);
            (this.invoke)(core::mem::transmute_copy(&sender), core::mem::transmute_copy(&e));
            windows_core::HRESULT(0)
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XamlReader(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(XamlReader, windows_core::IUnknown, windows_core::IInspectable);
impl XamlReader {
    pub fn Load(xaml: &str) -> windows_core::Result<windows_core::IInspectable> {
        Self::IXamlReaderStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Load)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(&windows_core::HSTRING::from(xaml)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub fn LoadWithInitialTemplateValidation(xaml: &str) -> windows_core::Result<windows_core::IInspectable> {
        Self::IXamlReaderStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).LoadWithInitialTemplateValidation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(&windows_core::HSTRING::from(xaml)),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IXamlReaderStatics<R, F: FnOnce(&IXamlReaderStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<XamlReader, IXamlReaderStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for XamlReader {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<Self, IXamlReader>();
}
unsafe impl windows_core::Interface for XamlReader {
    type Vtable = <IXamlReader as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IXamlReader as windows_core::Interface>::IID;
}
impl core::ops::Deref for XamlReader {
    type Target = IXamlReader;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for XamlReader {
    const NAME: &'static str = "Windows.UI.Xaml.Markup.XamlReader";
}
unsafe impl Send for XamlReader {}
unsafe impl Sync for XamlReader {}
