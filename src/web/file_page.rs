//! 文件管理器 Web 页面 HTML

/// 文件管理器 HTML 页面常量
pub const FILE_MANAGER_HTML: &str = r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>文件管理器</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background: #f5f5f5;
            min-height: 100vh;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
        }
        .header {
            background: #fff;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            margin-bottom: 20px;
        }
        .header h1 {
            color: #333;
            font-size: 24px;
            margin-bottom: 15px;
        }
        .breadcrumb {
            display: flex;
            align-items: center;
            gap: 5px;
            flex-wrap: wrap;
        }
        .breadcrumb a {
            color: #1890ff;
            text-decoration: none;
            padding: 4px 8px;
            border-radius: 4px;
        }
        .breadcrumb a:hover {
            background: #e6f7ff;
        }
        .breadcrumb span {
            color: #999;
        }
        .toolbar {
            display: flex;
            gap: 10px;
            margin-bottom: 20px;
            flex-wrap: wrap;
        }
        .btn {
            padding: 8px 16px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            display: inline-flex;
            align-items: center;
            gap: 5px;
            transition: all 0.3s;
        }
        .btn-primary {
            background: #1890ff;
            color: #fff;
        }
        .btn-primary:hover {
            background: #40a9ff;
        }
        .btn-success {
            background: #52c41a;
            color: #fff;
        }
        .btn-success:hover {
            background: #73d13d;
        }
        .btn-danger {
            background: #ff4d4f;
            color: #fff;
        }
        .btn-danger:hover {
            background: #ff7875;
        }
        .btn-default {
            background: #fff;
            color: #333;
            border: 1px solid #d9d9d9;
        }
        .btn-default:hover {
            color: #1890ff;
            border-color: #1890ff;
        }
        .main-content {
            display: flex;
            gap: 20px;
        }
        .file-list-container {
            flex: 1;
            background: #fff;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            overflow: hidden;
        }
        .file-list {
            width: 100%;
            border-collapse: collapse;
        }
        .file-list th {
            background: #fafafa;
            padding: 12px 16px;
            text-align: left;
            font-weight: 500;
            color: #333;
            border-bottom: 1px solid #f0f0f0;
        }
        .file-list td {
            padding: 12px 16px;
            border-bottom: 1px solid #f0f0f0;
        }
        .file-list tr:hover {
            background: #f5f5f5;
        }
        .file-list tr.selected {
            background: #e6f7ff;
        }
        .file-name {
            display: flex;
            align-items: center;
            gap: 10px;
            cursor: pointer;
        }
        .file-icon {
            font-size: 20px;
        }
        .file-icon.folder {
            color: #faad14;
        }
        .file-icon.file {
            color: #1890ff;
        }
        .file-icon.image {
            color: #52c41a;
        }
        .file-icon.video {
            color: #eb2f96;
        }
        .file-icon.audio {
            color: #722ed1;
        }
        .file-icon.code {
            color: #fa8c16;
        }
        .file-actions {
            display: flex;
            gap: 5px;
        }
        .file-actions button {
            padding: 4px 8px;
            font-size: 12px;
        }
        .preview-panel {
            width: 400px;
            background: #fff;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            overflow: hidden;
            display: none;
        }
        .preview-panel.active {
            display: block;
        }
        .preview-header {
            padding: 15px;
            background: #fafafa;
            border-bottom: 1px solid #f0f0f0;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .preview-header h3 {
            font-size: 14px;
            color: #333;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            flex: 1;
        }
        .preview-content {
            padding: 15px;
            max-height: 600px;
            overflow: auto;
        }
        .preview-content img {
            max-width: 100%;
            height: auto;
        }
        .preview-content video,
        .preview-content audio {
            width: 100%;
        }
        .preview-content pre {
            background: #f5f5f5;
            padding: 15px;
            border-radius: 4px;
            overflow-x: auto;
            font-size: 13px;
            line-height: 1.5;
        }
        .preview-content iframe {
            width: 100%;
            height: 500px;
            border: none;
        }
        .modal {
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0,0,0,0.5);
            display: none;
            justify-content: center;
            align-items: center;
            z-index: 1000;
        }
        .modal.active {
            display: flex;
        }
        .modal-content {
            background: #fff;
            padding: 24px;
            border-radius: 8px;
            min-width: 400px;
            max-width: 90%;
        }
        .modal-header {
            margin-bottom: 20px;
        }
        .modal-header h3 {
            color: #333;
        }
        .modal-body {
            margin-bottom: 20px;
        }
        .modal-body input {
            width: 100%;
            padding: 8px 12px;
            border: 1px solid #d9d9d9;
            border-radius: 4px;
            font-size: 14px;
        }
        .modal-body input:focus {
            outline: none;
            border-color: #1890ff;
            box-shadow: 0 0 0 2px rgba(24,144,255,0.2);
        }
        .modal-footer {
            display: flex;
            justify-content: flex-end;
            gap: 10px;
        }
        .upload-area {
            border: 2px dashed #d9d9d9;
            border-radius: 8px;
            padding: 40px;
            text-align: center;
            cursor: pointer;
            transition: all 0.3s;
        }
        .upload-area:hover,
        .upload-area.dragover {
            border-color: #1890ff;
            background: #e6f7ff;
        }
        .upload-area input {
            display: none;
        }
        .upload-icon {
            font-size: 48px;
            color: #1890ff;
            margin-bottom: 10px;
        }
        .empty-state {
            text-align: center;
            padding: 60px 20px;
            color: #999;
        }
        .empty-state .icon {
            font-size: 64px;
            margin-bottom: 20px;
        }
        .loading {
            text-align: center;
            padding: 40px;
            color: #999;
        }
        .toast {
            position: fixed;
            top: 20px;
            right: 20px;
            padding: 12px 24px;
            background: #333;
            color: #fff;
            border-radius: 4px;
            z-index: 2000;
            animation: fadeIn 0.3s;
        }
        .toast.success {
            background: #52c41a;
        }
        .toast.error {
            background: #ff4d4f;
        }
        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(-20px); }
            to { opacity: 1; transform: translateY(0); }
        }
        @media (max-width: 768px) {
            .main-content {
                flex-direction: column;
            }
            .preview-panel {
                width: 100%;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📁 文件管理器</h1>
            <div class="breadcrumb" id="breadcrumb">
                <a href="#" onclick="navigateTo('')">根目录</a>
            </div>
        </div>

        <div class="toolbar">
            <button class="btn btn-primary" onclick="showUploadModal()">📤 上传文件</button>
            <button class="btn btn-success" onclick="showCreateDirModal()">📁 新建文件夹</button>
            <button class="btn btn-default" onclick="refresh()">🔄 刷新</button>
        </div>

        <div class="main-content">
            <div class="file-list-container">
                <table class="file-list">
                    <thead>
                        <tr>
                            <th>名称</th>
                            <th>大小</th>
                            <th>修改时间</th>
                            <th>操作</th>
                        </tr>
                    </thead>
                    <tbody id="fileList">
                        <tr><td colspan="4" class="loading">加载中...</td></tr>
                    </tbody>
                </table>
            </div>

            <div class="preview-panel" id="previewPanel">
                <div class="preview-header">
                    <h3 id="previewTitle">预览</h3>
                    <button class="btn btn-default" onclick="closePreview()">✕</button>
                </div>
                <div class="preview-content" id="previewContent"></div>
            </div>
        </div>
    </div>

    <!-- 上传文件模态框 -->
    <div class="modal" id="uploadModal">
        <div class="modal-content">
            <div class="modal-header">
                <h3>上传文件</h3>
            </div>
            <div class="modal-body">
                <div class="upload-area" id="uploadArea">
                    <div class="upload-icon">📤</div>
                    <p>点击或拖拽文件到此处上传</p>
                    <input type="file" id="fileInput" multiple>
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-default" onclick="closeModal('uploadModal')">取消</button>
            </div>
        </div>
    </div>

    <!-- 新建文件夹模态框 -->
    <div class="modal" id="createDirModal">
        <div class="modal-content">
            <div class="modal-header">
                <h3>新建文件夹</h3>
            </div>
            <div class="modal-body">
                <input type="text" id="newDirName" placeholder="请输入文件夹名称">
            </div>
            <div class="modal-footer">
                <button class="btn btn-default" onclick="closeModal('createDirModal')">取消</button>
                <button class="btn btn-primary" onclick="createDir()">创建</button>
            </div>
        </div>
    </div>

    <!-- 重命名模态框 -->
    <div class="modal" id="renameModal">
        <div class="modal-content">
            <div class="modal-header">
                <h3>重命名</h3>
            </div>
            <div class="modal-body">
                <input type="text" id="newName" placeholder="请输入新名称">
            </div>
            <div class="modal-footer">
                <button class="btn btn-default" onclick="closeModal('renameModal')">取消</button>
                <button class="btn btn-primary" onclick="doRename()">确定</button>
            </div>
        </div>
    </div>

    <script>
        let currentPath = '';
        let renameTarget = '';

        // 初始化
        document.addEventListener('DOMContentLoaded', () => {
            loadFiles();
            setupUpload();
        });

        // 加载文件列表
        async function loadFiles() {
            const tbody = document.getElementById('fileList');
            tbody.innerHTML = '<tr><td colspan="4" class="loading">加载中...</td></tr>';

            try {
                const response = await fetch(`/api/file/list?path=${encodeURIComponent(currentPath)}`);
                const result = await response.json();

                if (result.state === 0 && result.data) {
                    renderFiles(result.data);
                    updateBreadcrumb();
                } else {
                    tbody.innerHTML = `<tr><td colspan="4" class="empty-state"><div class="icon">⚠️</div><p>${result.message}</p></td></tr>`;
                }
            } catch (error) {
                tbody.innerHTML = `<tr><td colspan="4" class="empty-state"><div class="icon">❌</div><p>加载失败: ${error.message}</p></td></tr>`;
            }
        }

        // 渲染文件列表
        function renderFiles(files) {
            const tbody = document.getElementById('fileList');

            if (files.length === 0) {
                tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><div class="icon">📂</div><p>文件夹为空</p></td></tr>';
                return;
            }

            tbody.innerHTML = files.map(file => `
                <tr>
                    <td>
                        <div class="file-name" onclick="${file.is_dir ? `navigateTo('${file.path}')` : `previewFile('${file.path}', '${file.name}')`}">
                            <span class="file-icon ${getFileClass(file)}">${getFileIcon(file)}</span>
                            <span>${file.name}</span>
                        </div>
                    </td>
                    <td>${file.is_dir ? '-' : formatSize(file.size)}</td>
                    <td>${file.modified || '-'}</td>
                    <td class="file-actions">
                        ${!file.is_dir ? `<button class="btn btn-default" onclick="downloadFile('${file.path}')">下载</button>` : ''}
                        <button class="btn btn-default" onclick="showRenameModal('${file.path}', '${file.name}')">重命名</button>
                        <button class="btn btn-danger" onclick="deleteFile('${file.path}', '${file.name}')">删除</button>
                    </td>
                </tr>
            `).join('');
        }

        // 获取文件图标
        function getFileIcon(file) {
            if (file.is_dir) return '📁';
            const ext = file.name.split('.').pop().toLowerCase();
            const icons = {
                // 图片
                'png': '🖼️', 'jpg': '🖼️', 'jpeg': '🖼️', 'gif': '🖼️', 'webp': '🖼️', 'svg': '🖼️', 'bmp': '🖼️',
                // 视频
                'mp4': '🎬', 'webm': '🎬', 'avi': '🎬', 'mkv': '🎬', 'mov': '🎬',
                // 音频
                'mp3': '🎵', 'wav': '🎵', 'flac': '🎵', 'aac': '🎵', 'm4a': '🎵',
                // 文档
                'pdf': '📕', 'doc': '📘', 'docx': '📘', 'xls': '📗', 'xlsx': '📗', 'ppt': '📙', 'pptx': '📙',
                // 代码
                'js': '📜', 'ts': '📜', 'py': '🐍', 'rs': '🦀', 'go': '🐹', 'java': '☕', 'c': '📜', 'cpp': '📜', 'h': '📜',
                'html': '🌐', 'css': '🎨', 'json': '📋', 'xml': '📋', 'yaml': '📋', 'yml': '📋', 'toml': '📋',
                // 文本
                'txt': '📄', 'md': '📝', 'log': '📄',
                // 压缩
                'zip': '📦', 'tar': '📦', 'gz': '📦', 'rar': '📦', '7z': '📦',
            };
            return icons[ext] || '📄';
        }

        // 获取文件类型 class
        function getFileClass(file) {
            if (file.is_dir) return 'folder';
            const ext = file.name.split('.').pop().toLowerCase();
            if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp'].includes(ext)) return 'image';
            if (['mp4', 'webm', 'avi', 'mkv', 'mov'].includes(ext)) return 'video';
            if (['mp3', 'wav', 'flac', 'aac', 'm4a'].includes(ext)) return 'audio';
            if (['js', 'ts', 'py', 'rs', 'go', 'java', 'c', 'cpp', 'h', 'html', 'css', 'json', 'xml', 'yaml', 'yml'].includes(ext)) return 'code';
            return 'file';
        }

        // 格式化文件大小
        function formatSize(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }

        // 导航到目录
        function navigateTo(path) {
            currentPath = path;
            loadFiles();
            closePreview();
        }

        // 更新面包屑导航
        function updateBreadcrumb() {
            const breadcrumb = document.getElementById('breadcrumb');
            const parts = currentPath.split('/').filter(p => p);

            let html = '<a href="#" onclick="navigateTo(\'\')">根目录</a>';
            let path = '';

            for (const part of parts) {
                path += (path ? '/' : '') + part;
                html += ` <span>/</span> <a href="#" onclick="navigateTo('${path}')">${part}</a>`;
            }

            breadcrumb.innerHTML = html;
        }

        // 预览文件
        async function previewFile(path, name) {
            const panel = document.getElementById('previewPanel');
            const title = document.getElementById('previewTitle');
            const content = document.getElementById('previewContent');

            panel.classList.add('active');
            title.textContent = name;
            content.innerHTML = '<div class="loading">加载中...</div>';

            const ext = name.split('.').pop().toLowerCase();

            // 图片预览
            if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp'].includes(ext)) {
                content.innerHTML = `<img src="/api/file/preview?path=${encodeURIComponent(path)}" alt="${name}">`;
                return;
            }

            // 视频预览
            if (['mp4', 'webm', 'ogg'].includes(ext)) {
                content.innerHTML = `<video controls><source src="/api/file/preview?path=${encodeURIComponent(path)}" type="video/${ext}"></video>`;
                return;
            }

            // 音频预览
            if (['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a'].includes(ext)) {
                const type = ext === 'mp3' ? 'mpeg' : ext;
                content.innerHTML = `<audio controls><source src="/api/file/preview?path=${encodeURIComponent(path)}" type="audio/${type}"></audio>`;
                return;
            }

            // PDF 预览
            if (ext === 'pdf') {
                content.innerHTML = `<iframe src="/api/file/preview?path=${encodeURIComponent(path)}"></iframe>`;
                return;
            }

            // 文本预览
            try {
                const response = await fetch(`/api/file/view?path=${encodeURIComponent(path)}`);
                const result = await response.json();

                if (result.state === 0 && result.data) {
                    if (result.message === 'base64') {
                        content.innerHTML = '<p style="color:#999">二进制文件，无法预览文本内容</p>';
                    } else {
                        content.innerHTML = `<pre>${escapeHtml(result.data)}</pre>`;
                    }
                } else {
                    content.innerHTML = `<p style="color:#ff4d4f">${result.message}</p>`;
                }
            } catch (error) {
                content.innerHTML = `<p style="color:#ff4d4f">预览失败: ${error.message}</p>`;
            }
        }

        // 关闭预览
        function closePreview() {
            document.getElementById('previewPanel').classList.remove('active');
        }

        // 下载文件
        function downloadFile(path) {
            window.open(`/api/file/download?path=${encodeURIComponent(path)}`, '_blank');
        }

        // 删除文件
        async function deleteFile(path, name) {
            if (!confirm(`确定要删除 "${name}" 吗？`)) return;

            try {
                const response = await fetch('/api/file/delete', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ path })
                });
                const result = await response.json();

                if (result.state === 0) {
                    showToast('删除成功', 'success');
                    loadFiles();
                    closePreview();
                } else {
                    showToast(result.message, 'error');
                }
            } catch (error) {
                showToast('删除失败: ' + error.message, 'error');
            }
        }

        // 显示上传模态框
        function showUploadModal() {
            document.getElementById('uploadModal').classList.add('active');
        }

        // 显示新建文件夹模态框
        function showCreateDirModal() {
            document.getElementById('newDirName').value = '';
            document.getElementById('createDirModal').classList.add('active');
        }

        // 显示重命名模态框
        function showRenameModal(path, name) {
            renameTarget = path;
            document.getElementById('newName').value = name;
            document.getElementById('renameModal').classList.add('active');
        }

        // 关闭模态框
        function closeModal(id) {
            document.getElementById(id).classList.remove('active');
        }

        // 创建文件夹
        async function createDir() {
            const name = document.getElementById('newDirName').value.trim();
            if (!name) {
                showToast('请输入文件夹名称', 'error');
                return;
            }

            const path = currentPath ? `${currentPath}/${name}` : name;

            try {
                const response = await fetch('/api/file/mkdir', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ path })
                });
                const result = await response.json();

                if (result.state === 0) {
                    showToast('创建成功', 'success');
                    closeModal('createDirModal');
                    loadFiles();
                } else {
                    showToast(result.message, 'error');
                }
            } catch (error) {
                showToast('创建失败: ' + error.message, 'error');
            }
        }

        // 执行重命名
        async function doRename() {
            const newName = document.getElementById('newName').value.trim();
            if (!newName) {
                showToast('请输入新名称', 'error');
                return;
            }

            const oldPath = renameTarget;
            const parts = oldPath.split('/');
            parts[parts.length - 1] = newName;
            const newPath = parts.join('/');

            try {
                const response = await fetch('/api/file/rename', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ old_path: oldPath, new_path: newPath })
                });
                const result = await response.json();

                if (result.state === 0) {
                    showToast('重命名成功', 'success');
                    closeModal('renameModal');
                    loadFiles();
                } else {
                    showToast(result.message, 'error');
                }
            } catch (error) {
                showToast('重命名失败: ' + error.message, 'error');
            }
        }

        // 设置上传功能
        function setupUpload() {
            const area = document.getElementById('uploadArea');
            const input = document.getElementById('fileInput');

            area.addEventListener('click', () => input.click());

            area.addEventListener('dragover', (e) => {
                e.preventDefault();
                area.classList.add('dragover');
            });

            area.addEventListener('dragleave', () => {
                area.classList.remove('dragover');
            });

            area.addEventListener('drop', (e) => {
                e.preventDefault();
                area.classList.remove('dragover');
                const files = e.dataTransfer.files;
                if (files.length > 0) {
                    uploadFiles(files);
                }
            });

            input.addEventListener('change', () => {
                if (input.files.length > 0) {
                    uploadFiles(input.files);
                }
            });
        }

        // 上传文件
        async function uploadFiles(files) {
            const formData = new FormData();
            for (const file of files) {
                formData.append('file', file);
            }

            try {
                const response = await fetch(`/api/file/upload?path=${encodeURIComponent(currentPath)}`, {
                    method: 'POST',
                    body: formData
                });
                const result = await response.json();

                if (result.state === 0) {
                    showToast(`上传成功: ${result.data.length} 个文件`, 'success');
                    closeModal('uploadModal');
                    loadFiles();
                } else {
                    showToast(result.message, 'error');
                }
            } catch (error) {
                showToast('上传失败: ' + error.message, 'error');
            }
        }

        // 刷新
        function refresh() {
            loadFiles();
        }

        // 显示提示
        function showToast(message, type = 'info') {
            const toast = document.createElement('div');
            toast.className = `toast ${type}`;
            toast.textContent = message;
            document.body.appendChild(toast);

            setTimeout(() => {
                toast.remove();
            }, 3000);
        }

        // HTML 转义
        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }
    </script>
</body>
</html>"##;
