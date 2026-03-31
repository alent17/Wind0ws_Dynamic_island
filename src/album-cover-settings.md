# 专辑封面设置功能说明

## 设置选项

已在 Settings.svelte 中添加两个新设置：

1. **enable_hd_cover** (默认：true)
   - 开启：从网络获取高清专辑封面
   - 关闭：直接使用 SMTC 提供的封面

2. **enable_pixel_art** (默认：false)
   - 开启：将封面像素化显示
   - 关闭：正常显示封面

## 组合效果

| HD 获取 | 像素化 | 效果 |
|---------|--------|------|
| ✅ 开启 | ✅ 开启 | 高清图的像素化版本（最佳质量） |
| ✅ 开启 | ❌ 关闭 | 正常高清图 |
| ❌ 关闭 | ✅ 开启 | SMTC 图的像素化版本 |
| ❌ 关闭 | ❌ 关闭 | 正常 SMTC 图 |

## 实现逻辑

FloatingWindow.svelte 已添加：
- `enableHDCover` 状态变量
- `enablePixelArt` 状态变量
- 在 onMount 中从设置加载
- `$effect` 根据 `enablePixelArt` 决定渲染方式

## 下一步

需要修改图片加载逻辑，根据 `enableHDCover` 决定是否调用 `fetchHighResCover`。
