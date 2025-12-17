//! 鏂囦欢绠＄悊鍣?Web 椤甸潰 HTML

/// 鏂囦欢绠＄悊鍣?HTML 椤甸潰甯搁噺
pub const FILE_MANAGER_HTML: &str = r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>鏂囦欢绠＄悊鍣?/title>
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
            <h1>馃搧 鏂囦欢绠＄悊鍣?/h1>
            <div class="breadcrumb" id="breadcrumb">
                <a href="#" onclick="navigateTo('')">鏍圭洰褰?/a>
            </div>
        </div>

        <div class="toolbar">
            <button class="btn btn-primary" onclick="showUploadModal()">馃摛 涓婁紶鏂囦欢</button>
            <button class="btn btn-success" onclick="showCreateDirModal()">馃搧 鏂板缓鏂囦欢澶?/button>
            <button class="btn btn-default" onclick="refresh()">馃攧 鍒锋柊</button>
        </div>

        <div class="main-content">
            <div class="file-list-container">
                <table class="file-list">
                    <thead>
                        <tr>
                            <th>鍚嶇О</th>
                            <th>澶у皬</th>
                            <th>淇敼鏃堕棿</th>
                            <th>鎿嶄綔</th>
                        </tr>
                    </thead>
                    <tbody id="fileList">
                        <tr><td colspan="4" class="loading">鍔犺浇涓?..</td></tr>
                    </tbody>
                </table>
            </div>

            <div class="preview-panel" id="previewPanel">
                <div class="preview-header">
                    <h3 id="previewTitle">棰勮</h3>
                    <button class="btn btn-default" onclick="closePreview()">鉁?/button>
                </div>
                <div class="preview-content" id="previewContent"></div>
            </div>
        </div>
    </div>

    <!-- 涓婁紶鏂囦欢妯℃€佹 -->
    <div class="modal" id="uploadModal">
        <div class="modal-content">
            <div class="modal-header">
                <h3>涓婁紶鏂囦欢</h3>
            </div>
            <div class="modal-body">
                <div class="upload-area" id="uploadArea">
                    <div class="upload-icon">馃摛</div>
                    <p>鐐瑰嚮鎴栨嫋鎷芥枃浠跺埌姝ゅ涓婁紶</p>
                    <input type="file" id="fileInput" multiple>
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-default" onclick="closeModal('uploadModal')">鍙栨秷</button>
            </div>
        </div>
    </div>

    <!-- 鏂板缓鏂囦欢澶规ā鎬佹 -->
    <div class="modal" id="createDirModal">
        <div class="modal-content">
            <div class="modal-header">
                <h3>鏂板缓鏂囦欢澶?/h3>
            </div>
            <div class="modal-body">
                <input type="text" id="newDirName" placeholder="璇疯緭鍏ユ枃浠跺す鍚嶇О">
            </div>
            <div class="modal-footer">
                <button class="btn btn-default" onclick="closeModal('createDirModal')">鍙栨秷</button>
                <button class="btn btn-primary" onclick="createDir()">鍒涘缓</button>
            </div>
        </div>
    </div>

    <!-- 閲嶅懡鍚嶆ā鎬佹 -->
    <div class="modal" id="renameModal">
        <div class="modal-content">
            <div class="modal-header">
                <h3>閲嶅懡鍚?/h3>
            </div>
            <div class="modal-body">
                <input type="text" id="newName" placeholder="璇疯緭鍏ユ柊鍚嶇О">
            </div>
            <div class="modal-footer">
                <button class="btn btn-default" onclick="closeModal('renameModal')">鍙栨秷</button>
                <button class="btn btn-primary" onclick="doRename()">纭畾</button>
            </div>
        </div>
    </div>

    <script>
        let currentPath = '';
        let renameTarget = '';

        // 鍒濆鍖?
        document.addEventListener('DOMContentLoaded', () => {
            loadFiles();
            setupUpload();
        });

        // 鍔犺浇鏂囦欢鍒楄〃
        async function loadFiles() {
            const tbody = document.getElementById('fileList');
            tbody.innerHTML = '<tr><td colspan="4" class="loading">鍔犺浇涓?..</td></tr>';

            try {
                const response = await fetch(`/api/file/list?path=${encodeURIComponent(currentPath)}`);
                const result = await response.json();

                if (result.state === 0 && result.data) {
                    renderFiles(result.data);
                    updateBreadcrumb();
                } else {
                    tbody.innerHTML = `<tr><td colspan="4" class="empty-state"><div class="icon">鈿狅笍</div><p>${result.message}</p></td></tr>`;
                }
            } catch (error) {
                tbody.innerHTML = `<tr><td colspan="4" class="empty-state"><div class="icon">鉂?/div><p>鍔犺浇澶辫触: ${error.message}</p></td></tr>`;
            }
        }

        // 娓叉煋鏂囦欢鍒楄〃
        function renderFiles(files) {
            const tbody = document.getElementById('fileList');

            if (files.length === 0) {
                tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><div class="icon">馃搨</div><p>鏂囦欢澶逛负绌?/p></td></tr>';
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
                        ${!file.is_dir ? `<button class="btn btn-default" onclick="downloadFile('${file.path}')">涓嬭浇</button>` : ''}
                        <button class="btn btn-default" onclick="showRenameModal('${file.path}', '${file.name}')">閲嶅懡鍚?/button>
                        <button class="btn btn-danger" onclick="deleteFile('${file.path}', '${file.name}')">鍒犻櫎</button>
                    </td>
                </tr>
            `).join('');
        }

        // 鑾峰彇鏂囦欢鍥炬爣
        function getFileIcon(file) {
            if (file.is_dir) return '馃搧';
            const ext = file.name.split('.').pop().toLowerCase();
            const icons = {
                // 鍥剧墖
                'png': '馃柤锔?, 'jpg': '馃柤锔?, 'jpeg': '馃柤锔?, 'gif': '馃柤锔?, 'webp': '馃柤锔?, 'svg': '馃柤锔?, 'bmp': '馃柤锔?,
                // 瑙嗛
                'mp4': '馃幀', 'webm': '馃幀', 'avi': '馃幀', 'mkv': '馃幀', 'mov': '馃幀',
                // 闊抽
                'mp3': '馃幍', 'wav': '馃幍', 'flac': '馃幍', 'aac': '馃幍', 'm4a': '馃幍',
                // 鏂囨。
                'pdf': '馃摃', 'doc': '馃摌', 'docx': '馃摌', 'xls': '馃摋', 'xlsx': '馃摋', 'ppt': '馃摍', 'pptx': '馃摍',
                // 浠ｇ爜
                'js': '馃摐', 'ts': '馃摐', 'py': '馃悕', 'rs': '馃', 'go': '馃惞', 'java': '鈽?, 'c': '馃摐', 'cpp': '馃摐', 'h': '馃摐',
                'html': '馃寪', 'css': '馃帹', 'json': '馃搵', 'xml': '馃搵', 'yaml': '馃搵', 'yml': '馃搵', 'toml': '馃搵',
                // 鏂囨湰
                'txt': '馃搫', 'md': '馃摑', 'log': '馃搫',
                // 鍘嬬缉
                'zip': '馃摝', 'tar': '馃摝', 'gz': '馃摝', 'rar': '馃摝', '7z': '馃摝',
            };
            return icons[ext] || '馃搫';
        }

        // 鑾峰彇鏂囦欢绫诲瀷 class
        function getFileClass(file) {
            if (file.is_dir) return 'folder';
            const ext = file.name.split('.').pop().toLowerCase();
            if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp'].includes(ext)) return 'image';
            if (['mp4', 'webm', 'avi', 'mkv', 'mov'].includes(ext)) return 'video';
            if (['mp3', 'wav', 'flac', 'aac', 'm4a'].includes(ext)) return 'audio';
            if (['js', 'ts', 'py', 'rs', 'go', 'java', 'c', 'cpp', 'h', 'html', 'css', 'json', 'xml', 'yaml', 'yml'].includes(ext)) return 'code';
            return 'file';
        }

        // 鏍煎紡鍖栨枃浠跺ぇ灏?
        function formatSize(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }

        // 瀵艰埅鍒扮洰褰?
        function navigateTo(path) {
            currentPath = path;
            loadFiles();
            closePreview();
        }

        // 鏇存柊闈㈠寘灞戝鑸?
        function updateBreadcrumb() {
            const breadcrumb = document.getElementById('breadcrumb');
            const parts = currentPath.split('/').filter(p => p);

            let html = '<a href="#" onclick="navigateTo(\'\')">鏍圭洰褰?/a>';
            let path = '';

            for (const part of parts) {
                path += (path ? '/' : '') + part;
                html += ` <span>/</span> <a href="#" onclick="navigateTo('${path}')">${part}</a>`;
            }

            breadcrumb.innerHTML = html;
        }

        // 棰勮鏂囦欢
        async function previewFile(path, name) {
            const panel = document.getElementById('previewPanel');
            const title = document.getElementById('previewTitle');
            const content = document.getElementById('previewContent');

            panel.classList.add('active');
            title.textContent = name;
            content.innerHTML = '<div class="loading">鍔犺浇涓?..</div>';

            const ext = name.split('.').pop().toLowerCase();

            // 鍥剧墖棰勮
            if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp'].includes(ext)) {
                content.innerHTML = `<img src="/api/file/preview?path=${encodeURIComponent(path)}" alt="${name}">`;
                return;
            }

            // 瑙嗛棰勮
            if (['mp4', 'webm', 'ogg'].includes(ext)) {
                content.innerHTML = `<video controls><source src="/api/file/preview?path=${encodeURIComponent(path)}" type="video/${ext}"></video>`;
                return;
            }

            // 闊抽棰勮
            if (['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a'].includes(ext)) {
                const type = ext === 'mp3' ? 'mpeg' : ext;
                content.innerHTML = `<audio controls><source src="/api/file/preview?path=${encodeURIComponent(path)}" type="audio/${type}"></audio>`;
                return;
            }

            // PDF 棰勮
            if (ext === 'pdf') {
                content.innerHTML = `<iframe src="/api/file/preview?path=${encodeURIComponent(path)}"></iframe>`;
                return;
            }

            // 鏂囨湰棰勮
            try {
                const response = await fetch(`/api/file/view?path=${encodeURIComponent(path)}`);
                const result = await response.json();

                if (result.state === 0 && result.data) {
                    if (result.message === 'base64') {
                        content.innerHTML = '<p style="color:#999">浜岃繘鍒舵枃浠讹紝鏃犳硶棰勮鏂囨湰鍐呭</p>';
                    } else {
                        content.innerHTML = `<pre>${escapeHtml(result.data)}</pre>`;
                    }
                } else {
                    content.innerHTML = `<p style="color:#ff4d4f">${result.message}</p>`;
                }
            } catch (error) {
                content.innerHTML = `<p style="color:#ff4d4f">棰勮澶辫触: ${error.message}</p>`;
            }
        }

        // 鍏抽棴棰勮
        function closePreview() {
            document.getElementById('previewPanel').classList.remove('active');
        }

        // 涓嬭浇鏂囦欢
        function downloadFile(path) {
            window.open(`/api/file/download?path=${encodeURIComponent(path)}`, '_blank');
        }

        // 鍒犻櫎鏂囦欢
        async function deleteFile(path, name) {
            if (!confirm(`纭畾瑕佸垹闄?"${name}" 鍚楋紵`)) return;

            try {
                const response = await fetch('/api/file/delete', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ path })
                });
                const result = await response.json();

                if (result.state === 0) {
                    showToast('鍒犻櫎鎴愬姛', 'success');
                    loadFiles();
                    closePreview();
                } else {
                    showToast(result.message, 'error');
                }
            } catch (error) {
                showToast('鍒犻櫎澶辫触: ' + error.message, 'error');
            }
        }

        // 鏄剧ず涓婁紶妯℃€佹
        function showUploadModal() {
            document.getElementById('uploadModal').classList.add('active');
        }

        // 鏄剧ず鏂板缓鏂囦欢澶规ā鎬佹
        function showCreateDirModal() {
            document.getElementById('newDirName').value = '';
            document.getElementById('createDirModal').classList.add('active');
        }

        // 鏄剧ず閲嶅懡鍚嶆ā鎬佹
        function showRenameModal(path, name) {
            renameTarget = path;
            document.getElementById('newName').value = name;
            document.getElementById('renameModal').classList.add('active');
        }

        // 鍏抽棴妯℃€佹
        function closeModal(id) {
            document.getElementById(id).classList.remove('active');
        }

        // 鍒涘缓鏂囦欢澶?
        async function createDir() {
            const name = document.getElementById('newDirName').value.trim();
            if (!name) {
                showToast('璇疯緭鍏ユ枃浠跺す鍚嶇О', 'error');
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
                    showToast('鍒涘缓鎴愬姛', 'success');
                    closeModal('createDirModal');
                    loadFiles();
                } else {
                    showToast(result.message, 'error');
                }
            } catch (error) {
                showToast('鍒涘缓澶辫触: ' + error.message, 'error');
            }
        }

        // 鎵ц閲嶅懡鍚?
        async function doRename() {
            const newName = document.getElementById('newName').value.trim();
            if (!newName) {
                showToast('璇疯緭鍏ユ柊鍚嶇О', 'error');
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
                    showToast('閲嶅懡鍚嶆垚鍔?, 'success');
                    closeModal('renameModal');
                    loadFiles();
                } else {
                    showToast(result.message, 'error');
                }
            } catch (error) {
                showToast('閲嶅懡鍚嶅け璐? ' + error.message, 'error');
            }
        }

        // 璁剧疆涓婁紶鍔熻兘
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

        // 涓婁紶鏂囦欢
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
                    showToast(`涓婁紶鎴愬姛: ${result.data.length} 涓枃浠禶, 'success');
                    closeModal('uploadModal');
                    loadFiles();
                } else {
                    showToast(result.message, 'error');
                }
            } catch (error) {
                showToast('涓婁紶澶辫触: ' + error.message, 'error');
            }
        }

        // 鍒锋柊
        function refresh() {
            loadFiles();
        }

        // 鏄剧ず鎻愮ず
        function showToast(message, type = 'info') {
            const toast = document.createElement('div');
            toast.className = `toast ${type}`;
            toast.textContent = message;
            document.body.appendChild(toast);

            setTimeout(() => {
                toast.remove();
            }, 3000);
        }

        // HTML 杞箟
        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }
    </script>
</body>
</html>"##;

/// Debug Console HTML Page
pub const DEBUG_CONSOLE_HTML: &str = r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Debug Console</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Consolas', 'Monaco', monospace;
            background: linear-gradient(135deg, #0d1117 0%, #161b22 100%);
            min-height: 100vh;
            color: #c9d1d9;
        }
        .container { max-width: 1800px; margin: 0 auto; padding: 15px; }
        .header {
            background: rgba(88,166,255,0.1);
            padding: 15px 20px;
            border-radius: 8px;
            margin-bottom: 15px;
            border: 1px solid #30363d;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .header h1 { color: #58a6ff; font-size: 20px; }
        .header-actions { display: flex; gap: 10px; }
        .main-layout { display: grid; grid-template-columns: 350px 1fr 400px; gap: 15px; min-height: calc(100vh - 180px); }
        .panel {
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 8px;
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }
        .panel-header {
            background: #21262d;
            padding: 10px 15px;
            border-bottom: 1px solid #30363d;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .panel-header h2 { font-size: 13px; color: #8b949e; }
        .panel-content { flex: 1; overflow-y: auto; padding: 10px; }
        .btn {
            padding: 5px 12px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
            font-family: inherit;
        }
        .btn-primary { background: #238636; color: #fff; }
        .btn-primary:hover { background: #2ea043; }
        .btn-secondary { background: #21262d; color: #c9d1d9; border: 1px solid #30363d; }
        .btn-secondary:hover { background: #30363d; }
        .btn-warning { background: #9e6a03; color: #fff; }
        .btn-danger { background: #da3633; color: #fff; }
        .btn-info { background: #1f6feb; color: #fff; }
        .btn-sm { padding: 3px 8px; font-size: 11px; }
        .device-item {
            background: #0d1117;
            border: 1px solid #21262d;
            border-radius: 6px;
            padding: 10px;
            margin-bottom: 8px;
            cursor: pointer;
        }
        .device-item:hover { border-color: #58a6ff; }
        .device-item.selected { border-color: #58a6ff; background: rgba(88,166,255,0.1); }
        .device-name { font-weight: 600; color: #f0f6fc; margin-bottom: 4px; }
        .device-meta { font-size: 11px; color: #8b949e; display: flex; gap: 8px; flex-wrap: wrap; }
        .device-meta span { background: #21262d; padding: 2px 6px; border-radius: 3px; }
        .device-value { margin-top: 6px; font-size: 12px; color: #7ee787; }
        .scene-item {
            background: #0d1117;
            border: 1px solid #21262d;
            border-radius: 6px;
            padding: 10px;
            margin-bottom: 8px;
        }
        .scene-item:hover { border-color: #a371f7; }
        .scene-name { font-weight: 600; color: #f0f6fc; margin-bottom: 4px; }
        .scene-nodes { font-size: 11px; color: #8b949e; margin-bottom: 8px; }
        .scene-actions { display: flex; gap: 6px; }
        .control-section {
            background: #0d1117;
            border: 1px solid #21262d;
            border-radius: 6px;
            padding: 12px;
            margin-bottom: 12px;
        }
        .control-section h3 {
            font-size: 12px;
            color: #58a6ff;
            margin-bottom: 10px;
            padding-bottom: 8px;
            border-bottom: 1px solid #21262d;
        }
        .control-row {
            display: flex;
            align-items: center;
            gap: 10px;
            margin-bottom: 8px;
        }
        .control-row label { font-size: 12px; color: #8b949e; min-width: 70px; }
        .control-row input, .control-row select {
            flex: 1;
            padding: 6px 10px;
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 4px;
            color: #c9d1d9;
            font-size: 12px;
            font-family: inherit;
        }
        .control-row input:focus, .control-row select:focus {
            outline: none;
            border-color: #58a6ff;
        }
        .log-entry {
            padding: 8px 10px;
            margin-bottom: 6px;
            border-radius: 4px;
            border-left: 3px solid;
            background: rgba(255,255,255,0.02);
            font-size: 11px;
        }
        .log-entry.request { border-color: #58a6ff; }
        .log-entry.response { border-color: #7ee787; }
        .log-entry.error { border-color: #f85149; background: rgba(248,81,73,0.1); }
        .log-entry.info { border-color: #a371f7; }
        .log-header { display: flex; justify-content: space-between; margin-bottom: 4px; }
        .log-time { color: #6e7681; }
        .log-type { font-weight: 600; }
        .log-type.request { color: #58a6ff; }
        .log-type.response { color: #7ee787; }
        .log-type.error { color: #f85149; }
        .log-type.info { color: #a371f7; }
        .log-body {
            background: #0d1117;
            padding: 8px;
            border-radius: 4px;
            margin-top: 6px;
            white-space: pre-wrap;
            word-break: break-all;
            max-height: 150px;
            overflow-y: auto;
        }
        .status-dot {
            display: inline-block;
            width: 8px;
            height: 8px;
            border-radius: 50%;
            margin-right: 6px;
        }
        .status-dot.online { background: #7ee787; }
        .status-dot.offline { background: #f85149; }
        .status-dot.unknown { background: #8b949e; }
        .tabs { display: flex; border-bottom: 1px solid #21262d; background: #161b22; }
        .tab {
            padding: 10px 16px;
            font-size: 12px;
            color: #8b949e;
            cursor: pointer;
            border-bottom: 2px solid transparent;
        }
        .tab:hover { color: #c9d1d9; }
        .tab.active { color: #58a6ff; border-bottom-color: #58a6ff; }
        .tab-content { display: none; }
        .tab-content.active { display: block; }
        .stats { display: flex; gap: 15px; margin-bottom: 12px; }
        .stat-item {
            background: #0d1117;
            border: 1px solid #21262d;
            border-radius: 6px;
            padding: 10px 15px;
            text-align: center;
            flex: 1;
        }
        .stat-value { font-size: 24px; font-weight: 600; color: #58a6ff; }
        .stat-label { font-size: 11px; color: #8b949e; margin-top: 4px; }
        .batch-textarea {
            width: 100%;
            height: 80px;
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 4px;
            padding: 10px;
            color: #c9d1d9;
            font-family: inherit;
            font-size: 12px;
            resize: vertical;
        }
        ::-webkit-scrollbar { width: 6px; }
        ::-webkit-scrollbar-track { background: #0d1117; }
        ::-webkit-scrollbar-thumb { background: #30363d; border-radius: 3px; }
        @media (max-width: 1200px) {
            .main-layout { grid-template-columns: 1fr; }
            .panel { max-height: 400px; }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Debug Console</h1>
            <div class="header-actions">
                <button class="btn btn-secondary" onclick="loadConfig()">Refresh</button>
                <button class="btn btn-secondary" onclick="getAllNodeStates()">Get All States</button>
                <button class="btn btn-danger" onclick="clearLogs()">Clear Logs</button>
            </div>
        </div>

        <div class="stats">
            <div class="stat-item">
                <div class="stat-value" id="deviceCount">0</div>
                <div class="stat-label">Devices</div>
            </div>
            <div class="stat-item">
                <div class="stat-value" id="sceneCount">0</div>
                <div class="stat-label">Scenes</div>
            </div>
            <div class="stat-item">
                <div class="stat-value" id="channelCount">0</div>
                <div class="stat-label">Channels</div>
            </div>
            <div class="stat-item">
                <div class="stat-value" id="logCount">0</div>
                <div class="stat-label">Logs</div>
            </div>
        </div>

        <div class="main-layout">
            <div class="panel">
                <div class="tabs">
                    <div class="tab active" onclick="switchTab('devices')">Devices</div>
                    <div class="tab" onclick="switchTab('scenes')">Scenes</div>
                    <div class="tab" onclick="switchTab('channels')">Channels</div>
                </div>
                <div class="panel-content">
                    <div id="devicesTab" class="tab-content active">
                        <div id="deviceList"><div style="color:#8b949e;text-align:center;padding:20px;">Loading...</div></div>
                    </div>
                    <div id="scenesTab" class="tab-content">
                        <div id="sceneList"><div style="color:#8b949e;text-align:center;padding:20px;">Loading...</div></div>
                    </div>
                    <div id="channelsTab" class="tab-content">
                        <div id="channelList"><div style="color:#8b949e;text-align:center;padding:20px;">Loading...</div></div>
                    </div>
                </div>
            </div>

            <div class="panel">
                <div class="panel-header">
                    <h2>Control Panel</h2>
                </div>
                <div class="panel-content">
                    <div class="control-section">
                        <h3>Device Read/Write</h3>
                        <div class="control-row">
                            <label>Device ID:</label>
                            <input type="number" id="ctrlDeviceId" placeholder="Global ID">
                            <button class="btn btn-info btn-sm" onclick="readDevice()">Read</button>
                        </div>
                        <div class="control-row">
                            <label>Write Value:</label>
                            <input type="number" id="ctrlWriteValue" placeholder="Value">
                            <button class="btn btn-warning btn-sm" onclick="writeDevice()">Write</button>
                        </div>
                        <div class="control-row">
                            <label>Current:</label>
                            <input type="text" id="ctrlCurrentValue" readonly placeholder="--" style="background:#0d1117;">
                        </div>
                    </div>

                    <div class="control-section">
                        <h3>Batch Operations</h3>
                        <div class="control-row">
                            <label>Device IDs:</label>
                            <input type="text" id="batchIds" placeholder="1,2,3,4">
                        </div>
                        <div style="display:flex;gap:8px;margin-top:8px;">
                            <button class="btn btn-info" onclick="batchRead()">Batch Read</button>
                        </div>
                    </div>

                    <div class="control-section">
                        <h3>Scene Execution</h3>
                        <div class="control-row">
                            <label>Scene:</label>
                            <select id="sceneSelect" style="flex:1;">
                                <option value="">-- Select Scene --</option>
                            </select>
                            <button class="btn btn-primary" onclick="executeSelectedScene()">Execute</button>
                        </div>
                    </div>

                    <div class="control-section">
                        <h3>Custom API</h3>
                        <div class="control-row">
                            <label>Method:</label>
                            <select id="customMethod">
                                <option value="GET">GET</option>
                                <option value="POST" selected>POST</option>
                            </select>
                        </div>
                        <div class="control-row">
                            <label>Path:</label>
                            <input type="text" id="customPath" placeholder="/lspcapi/device/...">
                        </div>
                        <div style="margin-top:8px;">
                            <label style="font-size:12px;color:#8b949e;display:block;margin-bottom:4px;">Body (JSON):</label>
                            <textarea class="batch-textarea" id="customBody" placeholder='{"key": "value"}'></textarea>
                        </div>
                        <button class="btn btn-primary" onclick="executeCustomApi()" style="width:100%;margin-top:8px;">Send Request</button>
                    </div>
                </div>
            </div>

            <div class="panel">
                <div class="panel-header">
                    <h2>Debug Logs</h2>
                    <button class="btn btn-sm btn-secondary" onclick="exportLogs()">Export</button>
                </div>
                <div class="panel-content" id="logContent"></div>
            </div>
        </div>
    </div>

    <script>
        let devices = [];
        let scenes = [];
        let channels = [];
        let logs = [];
        let selectedDevice = null;

        document.addEventListener('DOMContentLoaded', () => {
            addLog('info', 'System Init', 'Debug console started');
            loadConfig();
        });

        function switchTab(tab) {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(t => t.classList.remove('active'));
            event.target.classList.add('active');
            document.getElementById(tab + 'Tab').classList.add('active');
        }

        function addLog(type, title, content) {
            const time = new Date().toLocaleTimeString();
            logs.unshift({ type, title, content, time });
            document.getElementById('logCount').textContent = logs.length;
            renderLogs();
        }

        function renderLogs() {
            const container = document.getElementById('logContent');
            container.innerHTML = logs.map(log => `
                <div class="log-entry ${log.type}">
                    <div class="log-header">
                        <span><span class="log-type ${log.type}">[${log.type.toUpperCase()}]</span> ${log.title}</span>
                        <span class="log-time">${log.time}</span>
                    </div>
                    <div class="log-body">${typeof log.content === 'object' ? JSON.stringify(log.content, null, 2) : log.content}</div>
                </div>
            `).join('') || '<div style="color:#8b949e;text-align:center;padding:40px;">No logs</div>';
        }

        function clearLogs() {
            logs = [];
            document.getElementById('logCount').textContent = 0;
            renderLogs();
        }

        function exportLogs() {
            const blob = new Blob([JSON.stringify(logs, null, 2)], {type: 'application/json'});
            const a = document.createElement('a');
            a.href = URL.createObjectURL(blob);
            a.download = 'debug-logs.json';
            a.click();
        }

        async function apiRequest(method, url, body = null) {
            const start = performance.now();
            addLog('request', `${method} ${url}`, body || '(no body)');
            try {
                const opts = { method, headers: { 'Content-Type': 'application/json' } };
                if (body && method !== 'GET') opts.body = JSON.stringify(body);
                const res = await fetch(url, opts);
                const result = await res.json();
                const ms = (performance.now() - start).toFixed(0);
                if (result.state === 0) {
                    addLog('response', `${method} ${url} (${ms}ms)`, result);
                } else {
                    addLog('error', `${method} ${url} failed`, result.message || result);
                }
                return result;
            } catch (e) {
                addLog('error', `${method} ${url} error`, e.message);
                throw e;
            }
        }

        async function loadConfig() {
            try {
                const result = await apiRequest('GET', '/lspcapi/device/config');
                if (result.state === 0 && result.data) {
                    devices = result.data.nodes || [];
                    scenes = result.data.scenes || [];
                    channels = result.data.channels || [];
                    document.getElementById('deviceCount').textContent = devices.length;
                    document.getElementById('sceneCount').textContent = scenes.length;
                    document.getElementById('channelCount').textContent = channels.length;
                    renderDevices();
                    renderScenes();
                    renderChannels();
                    updateSceneSelect();
                }
            } catch (e) {
                document.getElementById('deviceList').innerHTML = '<div style="color:#f85149;padding:20px;">Load failed</div>';
            }
        }

        function renderDevices() {
            const c = document.getElementById('deviceList');
            if (!devices.length) { c.innerHTML = '<div style="color:#8b949e;text-align:center;padding:20px;">No devices</div>'; return; }
            c.innerHTML = devices.map(d => `
                <div class="device-item ${selectedDevice === d.global_id ? 'selected' : ''}" onclick="selectDevice(${d.global_id})">
                    <div class="device-name"><span class="status-dot unknown"></span>${d.alias || 'Unnamed'}</div>
                    <div class="device-meta">
                        <span>ID: ${d.global_id}</span>
                        <span>CH: ${d.channel_id}</span>
                    </div>
                    <div class="device-value" id="device-value-${d.global_id}">Value: --</div>
                </div>
            `).join('');
        }

        function renderScenes() {
            const c = document.getElementById('sceneList');
            if (!scenes.length) { c.innerHTML = '<div style="color:#8b949e;text-align:center;padding:20px;">No scenes</div>'; return; }
            c.innerHTML = scenes.map(s => `
                <div class="scene-item">
                    <div class="scene-name">${s.name}</div>
                    <div class="scene-nodes">${s.nodes ? s.nodes.length : 0} node operations</div>
                    <div class="scene-actions">
                        <button class="btn btn-primary btn-sm" onclick="executeScene('${s.name}')">Execute</button>
                        <button class="btn btn-secondary btn-sm" onclick="addLog('info', 'Scene: ${s.name}', ${JSON.stringify(s)})">Detail</button>
                    </div>
                </div>
            `).join('');
        }

        function renderChannels() {
            const c = document.getElementById('channelList');
            if (!channels.length) { c.innerHTML = '<div style="color:#8b949e;text-align:center;padding:20px;">No channels</div>'; return; }
            c.innerHTML = channels.map(ch => `
                <div class="device-item">
                    <div class="device-name"><span class="status-dot ${ch.enable ? 'online' : 'offline'}"></span>Channel ${ch.channel_id}</div>
                    <div class="device-meta">
                        <span>Protocol: ${ch.statute}</span>
                        <span>${ch.enable ? 'Enabled' : 'Disabled'}</span>
                    </div>
                </div>
            `).join('');
        }

        function updateSceneSelect() {
            document.getElementById('sceneSelect').innerHTML = '<option value="">-- Select Scene --</option>' + scenes.map(s => `<option value="${s.name}">${s.name}</option>`).join('');
        }

        function selectDevice(id) {
            selectedDevice = id;
            document.getElementById('ctrlDeviceId').value = id;
            renderDevices();
            readDevice();
        }

        async function readDevice() {
            const id = parseInt(document.getElementById('ctrlDeviceId').value);
            if (isNaN(id)) { addLog('error', 'Read failed', 'Invalid device ID'); return; }
            try {
                const r = await apiRequest('POST', '/lspcapi/device/read', { global_id: id });
                if (r.state === 0) {
                    document.getElementById('ctrlCurrentValue').value = r.data;
                    const el = document.getElementById(`device-value-${id}`);
                    if (el) el.textContent = `Value: ${r.data}`;
                }
            } catch (e) {}
        }

        async function writeDevice() {
            const id = parseInt(document.getElementById('ctrlDeviceId').value);
            const val = parseInt(document.getElementById('ctrlWriteValue').value);
            if (isNaN(id)) { addLog('error', 'Write failed', 'Invalid device ID'); return; }
            if (isNaN(val)) { addLog('error', 'Write failed', 'Invalid value'); return; }
            try {
                await apiRequest('POST', '/lspcapi/device/write', { global_id: id, value: val });
                setTimeout(readDevice, 100);
            } catch (e) {}
        }

        async function batchRead() {
            const ids = document.getElementById('batchIds').value.split(',').map(s => parseInt(s.trim())).filter(n => !isNaN(n));
            if (!ids.length) { addLog('error', 'Batch read failed', 'No valid IDs'); return; }
            try {
                const r = await apiRequest('POST', '/lspcapi/device/readMany', { ids });
                if (r.state === 0 && r.data) {
                    r.data.forEach(item => {
                        const el = document.getElementById(`device-value-${item.id}`);
                        if (el) el.textContent = item.success ? `Value: ${item.value}` : `Error: ${item.error}`;
                    });
                }
            } catch (e) {}
        }

        async function executeScene(name) {
            addLog('info', 'Scene execution', `Executing: ${name}`);
            try { await apiRequest('POST', '/lspcapi/device/scene', { name }); } catch (e) {}
        }

        function executeSelectedScene() {
            const name = document.getElementById('sceneSelect').value;
            if (!name) { addLog('error', 'Scene failed', 'Select a scene first'); return; }
            executeScene(name);
        }

        async function executeCustomApi() {
            const method = document.getElementById('customMethod').value;
            const path = document.getElementById('customPath').value;
            let body = null;
            if (method !== 'GET') {
                try {
                    const s = document.getElementById('customBody').value;
                    if (s) body = JSON.parse(s);
                } catch (e) { addLog('error', 'Parse error', 'Invalid JSON body'); return; }
            }
            if (!path) { addLog('error', 'API failed', 'Enter a path'); return; }
            try { await apiRequest(method, path, body); } catch (e) {}
        }

        async function getAllNodeStates() {
            try {
                const r = await apiRequest('POST', '/lspcapi/device/getAllNodeStates', {});
                if (r.state === 0 && r.data) {
                    r.data.forEach(node => {
                        const el = document.getElementById(`device-value-${node.global_id}`);
                        if (el) el.textContent = `Value: ${node.current_value !== null ? node.current_value : '--'}`;
                    });
                }
            } catch (e) {}
        }
    </script>
</body>
</html>"##;

/// Config Manager HTML Page
pub const CONFIG_MANAGER_HTML: &str = r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Config Manager</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: 'Segoe UI', sans-serif; background: #0d1117; min-height: 100vh; color: #c9d1d9; }
        .container { max-width: 1400px; margin: 0 auto; padding: 20px; }
        .header { background: rgba(88,166,255,0.1); padding: 15px 20px; border-radius: 8px; margin-bottom: 20px; border: 1px solid #30363d; display: flex; justify-content: space-between; align-items: center; }
        .header h1 { color: #58a6ff; font-size: 22px; }
        .btn { padding: 8px 16px; border: none; border-radius: 6px; cursor: pointer; font-size: 13px; transition: all 0.2s; }
        .btn-primary { background: #238636; color: #fff; }
        .btn-primary:hover { background: #2ea043; }
        .btn-secondary { background: #21262d; color: #c9d1d9; border: 1px solid #30363d; }
        .btn-secondary:hover { background: #30363d; }
        .btn-danger { background: #da3633; color: #fff; }
        .btn-danger:hover { background: #f85149; }
        .btn-sm { padding: 4px 10px; font-size: 12px; }
        .tabs { display: flex; gap: 10px; margin-bottom: 20px; }
        .tab { padding: 10px 20px; background: #21262d; border: 1px solid #30363d; border-radius: 6px; cursor: pointer; color: #8b949e; }
        .tab.active { background: #58a6ff; color: #fff; border-color: #58a6ff; }
        .panel { background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 20px; }
        .table { width: 100%; border-collapse: collapse; }
        .table th, .table td { padding: 12px; text-align: left; border-bottom: 1px solid #21262d; }
        .table th { background: #21262d; color: #8b949e; font-weight: 500; }
        .table tr:hover { background: rgba(88,166,255,0.05); }
        .form-group { margin-bottom: 15px; }
        .form-group label { display: block; margin-bottom: 5px; color: #8b949e; font-size: 13px; }
        .form-group input, .form-group select, .form-group textarea { width: 100%; padding: 10px; background: #0d1117; border: 1px solid #30363d; border-radius: 6px; color: #c9d1d9; font-size: 14px; }
        .form-group input:focus, .form-group select:focus, .form-group textarea:focus { outline: none; border-color: #58a6ff; }
        .modal { display: none; position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); z-index: 1000; }
        .modal.show { display: flex; align-items: center; justify-content: center; }
        .modal-content { background: #161b22; border: 1px solid #30363d; border-radius: 12px; padding: 25px; min-width: 450px; max-width: 600px; max-height: 80vh; overflow-y: auto; }
        .modal-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
        .modal-header h2 { color: #f0f6fc; font-size: 18px; }
        .modal-close { background: none; border: none; color: #8b949e; font-size: 24px; cursor: pointer; }
        .modal-footer { margin-top: 20px; display: flex; gap: 10px; justify-content: flex-end; }
        .alert { padding: 12px 16px; border-radius: 6px; margin-bottom: 15px; }
        .alert-success { background: rgba(35,134,54,0.2); border: 1px solid #238636; color: #7ee787; }
        .alert-error { background: rgba(218,54,51,0.2); border: 1px solid #da3633; color: #f85149; }
        .actions { display: flex; gap: 6px; }
        .badge { padding: 3px 8px; border-radius: 12px; font-size: 11px; }
        .badge-green { background: rgba(35,134,54,0.3); color: #7ee787; }
        .badge-gray { background: rgba(110,118,129,0.3); color: #8b949e; }
        .scene-nodes { font-size: 12px; color: #8b949e; }
        .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Config Manager</h1>
            <div>
                <button class="btn btn-secondary" onclick="loadConfig()">Reload</button>
                <button class="btn btn-primary" onclick="saveConfig()">Save Config</button>
                <a href="/lspcapi/debug" class="btn btn-secondary">Debug Console</a>
            </div>
        </div>
        <div id="alert"></div>
        <div class="tabs">
            <div class="tab active" onclick="showTab('nodes')">Devices</div>
            <div class="tab" onclick="showTab('scenes')">Scenes</div>
            <div class="tab" onclick="showTab('channels')">Channels</div>
        </div>
        <div id="nodesPanel" class="panel">
            <div class="toolbar">
                <h3>Device Nodes</h3>
                <button class="btn btn-primary btn-sm" onclick="showAddNode()">+ Add Device</button>
            </div>
            <table class="table"><thead><tr><th>ID</th><th>Alias</th><th>Channel</th><th>Actions</th></tr></thead><tbody id="nodesList"></tbody></table>
        </div>
        <div id="scenesPanel" class="panel" style="display:none;">
            <div class="toolbar">
                <h3>Scenes</h3>
                <button class="btn btn-primary btn-sm" onclick="showAddScene()">+ Add Scene</button>
            </div>
            <table class="table"><thead><tr><th>Name</th><th>Nodes</th><th>Actions</th></tr></thead><tbody id="scenesList"></tbody></table>
        </div>
        <div id="channelsPanel" class="panel" style="display:none;">
            <div class="toolbar">
                <h3>Channels</h3>
                <button class="btn btn-primary btn-sm" onclick="showAddChannel()">+ Add Channel</button>
            </div>
            <table class="table"><thead><tr><th>ID</th><th>Protocol</th><th>Status</th><th>Description</th><th>Actions</th></tr></thead><tbody id="channelsList"></tbody></table>
        </div>
    </div>
    <div id="nodeModal" class="modal">
        <div class="modal-content">
            <div class="modal-header"><h2 id="nodeModalTitle">Add Device</h2><button class="modal-close" onclick="closeModal('nodeModal')">&times;</button></div>
            <div class="form-group"><label>Global ID</label><input type="number" id="nodeId" placeholder="1"></div>
            <div class="form-group"><label>Channel ID</label><input type="number" id="nodeChannel" placeholder="1"></div>
            <div class="form-group"><label>Device ID (in channel)</label><input type="number" id="nodeDeviceId" placeholder="1"></div>
            <div class="form-group"><label>Alias</label><input type="text" id="nodeAlias" placeholder="Device name"></div>
            <div class="modal-footer"><button class="btn btn-secondary" onclick="closeModal('nodeModal')">Cancel</button><button class="btn btn-primary" onclick="saveNode()">Save</button></div>
        </div>
    </div>
    <div id="sceneModal" class="modal">
        <div class="modal-content">
            <div class="modal-header"><h2 id="sceneModalTitle">Add Scene</h2><button class="modal-close" onclick="closeModal('sceneModal')">&times;</button></div>
            <div class="form-group"><label>Scene Name</label><input type="text" id="sceneName" placeholder="Scene name"></div>
            <div class="form-group"><label>Nodes (JSON array)</label><textarea id="sceneNodes" rows="6" placeholder='[{"id": 1, "value": 100, "delay": 0}]'></textarea></div>
            <div class="modal-footer"><button class="btn btn-secondary" onclick="closeModal('sceneModal')">Cancel</button><button class="btn btn-primary" onclick="saveScene()">Save</button></div>
        </div>
    </div>
    <div id="channelModal" class="modal">
        <div class="modal-content">
            <div class="modal-header"><h2 id="channelModalTitle">Add Channel</h2><button class="modal-close" onclick="closeModal('channelModal')">&times;</button></div>
            <div class="form-group"><label>Channel ID</label><input type="number" id="channelId" placeholder="1"></div>
            <div class="form-group"><label>Protocol</label><select id="channelProtocol"><option value="mock">Mock</option><option value="modbus">Modbus</option><option value="pjlink">PJLink</option><option value="novastar">Novastar</option><option value="custom">Custom</option></select></div>
            <div class="form-group"><label>Enabled</label><select id="channelEnable"><option value="true">Yes</option><option value="false">No</option></select></div>
            <div class="form-group"><label>Description</label><input type="text" id="channelDesc" placeholder="Channel description"></div>
            <div class="form-group"><label>Arguments (JSON)</label><textarea id="channelArgs" rows="4" placeholder='{"delay_ms": 50}'></textarea></div>
            <div class="modal-footer"><button class="btn btn-secondary" onclick="closeModal('channelModal')">Cancel</button><button class="btn btn-primary" onclick="saveChannel()">Save</button></div>
        </div>
    </div>
    <script>
        let config = { nodes: [], scenes: [], channels: [], web_server: { port: 18080 } };
        let editIndex = -1;
        document.addEventListener('DOMContentLoaded', loadConfig);
        function showTab(tab) {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            event.target.classList.add('active');
            document.getElementById('nodesPanel').style.display = tab === 'nodes' ? 'block' : 'none';
            document.getElementById('scenesPanel').style.display = tab === 'scenes' ? 'block' : 'none';
            document.getElementById('channelsPanel').style.display = tab === 'channels' ? 'block' : 'none';
        }
        function showAlert(msg, type) {
            document.getElementById('alert').innerHTML = `<div class="alert alert-${type}">${msg}</div>`;
            setTimeout(() => document.getElementById('alert').innerHTML = '', 3000);
        }
        async function loadConfig() {
            try {
                const r = await fetch('/lspcapi/device/config');
                const data = await r.json();
                if (data.state === 0) {
                    config.nodes = data.data.nodes || [];
                    config.scenes = data.data.scenes || [];
                    config.channels = data.data.channels || [];
                    renderAll();
                    showAlert('Config loaded', 'success');
                }
            } catch (e) { showAlert('Load failed: ' + e.message, 'error'); }
        }
        async function saveConfig() {
            try {
                const fullConfig = { web_server: config.web_server, channels: config.channels, nodes: config.nodes, scenes: config.scenes };
                const r = await fetch('/lspcapi/config/save', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify(fullConfig) });
                const data = await r.json();
                if (data.state === 0) { showAlert('Config saved! Restart server to apply.', 'success'); }
                else { showAlert('Save failed: ' + data.message, 'error'); }
            } catch (e) { showAlert('Save error: ' + e.message, 'error'); }
        }
        function renderAll() { renderNodes(); renderScenes(); renderChannels(); }
        function renderNodes() {
            document.getElementById('nodesList').innerHTML = config.nodes.map((n, i) => `<tr><td>${n.global_id}</td><td>${n.alias}</td><td>${n.channel_id}</td><td class="actions"><button class="btn btn-secondary btn-sm" onclick="editNode(${i})">Edit</button><button class="btn btn-danger btn-sm" onclick="deleteNode(${i})">Delete</button></td></tr>`).join('') || '<tr><td colspan="4" style="text-align:center;color:#8b949e;">No devices</td></tr>';
        }
        function renderScenes() {
            document.getElementById('scenesList').innerHTML = config.scenes.map((s, i) => `<tr><td>${s.name}</td><td class="scene-nodes">${s.nodes ? s.nodes.length + ' ops' : '0 ops'}</td><td class="actions"><button class="btn btn-secondary btn-sm" onclick="editScene(${i})">Edit</button><button class="btn btn-danger btn-sm" onclick="deleteScene(${i})">Delete</button></td></tr>`).join('') || '<tr><td colspan="3" style="text-align:center;color:#8b949e;">No scenes</td></tr>';
        }
        function renderChannels() {
            document.getElementById('channelsList').innerHTML = config.channels.map((c, i) => `<tr><td>${c.channel_id}</td><td>${c.statute || c.protocol || 'mock'}</td><td><span class="badge ${c.enable ? 'badge-green' : 'badge-gray'}">${c.enable ? 'Enabled' : 'Disabled'}</span></td><td>${c.description || '-'}</td><td class="actions"><button class="btn btn-secondary btn-sm" onclick="editChannel(${i})">Edit</button><button class="btn btn-danger btn-sm" onclick="deleteChannel(${i})">Delete</button></td></tr>`).join('') || '<tr><td colspan="5" style="text-align:center;color:#8b949e;">No channels</td></tr>';
        }
        function closeModal(id) { document.getElementById(id).classList.remove('show'); editIndex = -1; }
        function showAddNode() { editIndex = -1; document.getElementById('nodeModalTitle').textContent = 'Add Device'; document.getElementById('nodeId').value = ''; document.getElementById('nodeChannel').value = ''; document.getElementById('nodeDeviceId').value = ''; document.getElementById('nodeAlias').value = ''; document.getElementById('nodeModal').classList.add('show'); }
        function editNode(i) { editIndex = i; const n = config.nodes[i]; document.getElementById('nodeModalTitle').textContent = 'Edit Device'; document.getElementById('nodeId').value = n.global_id; document.getElementById('nodeChannel').value = n.channel_id; document.getElementById('nodeDeviceId').value = n.id; document.getElementById('nodeAlias').value = n.alias; document.getElementById('nodeModal').classList.add('show'); }
        function saveNode() { const node = { global_id: parseInt(document.getElementById('nodeId').value), channel_id: parseInt(document.getElementById('nodeChannel').value), id: parseInt(document.getElementById('nodeDeviceId').value), alias: document.getElementById('nodeAlias').value }; if (editIndex >= 0) config.nodes[editIndex] = node; else config.nodes.push(node); closeModal('nodeModal'); renderNodes(); }
        function deleteNode(i) { if (confirm('Delete this device?')) { config.nodes.splice(i, 1); renderNodes(); } }
        function showAddScene() { editIndex = -1; document.getElementById('sceneModalTitle').textContent = 'Add Scene'; document.getElementById('sceneName').value = ''; document.getElementById('sceneNodes').value = '[]'; document.getElementById('sceneModal').classList.add('show'); }
        function editScene(i) { editIndex = i; const s = config.scenes[i]; document.getElementById('sceneModalTitle').textContent = 'Edit Scene'; document.getElementById('sceneName').value = s.name; document.getElementById('sceneNodes').value = JSON.stringify(s.nodes || [], null, 2); document.getElementById('sceneModal').classList.add('show'); }
        function saveScene() { try { const scene = { name: document.getElementById('sceneName').value, nodes: JSON.parse(document.getElementById('sceneNodes').value) }; if (editIndex >= 0) config.scenes[editIndex] = scene; else config.scenes.push(scene); closeModal('sceneModal'); renderScenes(); } catch (e) { showAlert('Invalid JSON: ' + e.message, 'error'); } }
        function deleteScene(i) { if (confirm('Delete this scene?')) { config.scenes.splice(i, 1); renderScenes(); } }
        function showAddChannel() { editIndex = -1; document.getElementById('channelModalTitle').textContent = 'Add Channel'; document.getElementById('channelId').value = ''; document.getElementById('channelProtocol').value = 'mock'; document.getElementById('channelEnable').value = 'true'; document.getElementById('channelDesc').value = ''; document.getElementById('channelArgs').value = '{}'; document.getElementById('channelModal').classList.add('show'); }
        function editChannel(i) { editIndex = i; const c = config.channels[i]; document.getElementById('channelModalTitle').textContent = 'Edit Channel'; document.getElementById('channelId').value = c.channel_id; document.getElementById('channelProtocol').value = c.statute || c.protocol || 'mock'; document.getElementById('channelEnable').value = c.enable ? 'true' : 'false'; document.getElementById('channelDesc').value = c.description || ''; document.getElementById('channelArgs').value = JSON.stringify(c.arguments || {}, null, 2); document.getElementById('channelModal').classList.add('show'); }
        function saveChannel() { try { const ch = { channel_id: parseInt(document.getElementById('channelId').value), statute: document.getElementById('channelProtocol').value, enable: document.getElementById('channelEnable').value === 'true', description: document.getElementById('channelDesc').value, arguments: JSON.parse(document.getElementById('channelArgs').value || '{}') }; if (editIndex >= 0) config.channels[editIndex] = ch; else config.channels.push(ch); closeModal('channelModal'); renderChannels(); } catch (e) { showAlert('Invalid JSON: ' + e.message, 'error'); } }
        function deleteChannel(i) { if (confirm('Delete this channel?')) { config.channels.splice(i, 1); renderChannels(); } }
    </script>
</body>
</html>"##;
