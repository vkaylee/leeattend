const ExcelJS = require('exceljs');
const path = require('path');
const fs = require('fs');

async function generateTemplate() {
    const workbook = new ExcelJS.Workbook();
    const sheet = workbook.addWorksheet('Employee Import');

    // Define columns
    sheet.columns = [
        { header: 'employee_code', key: 'id', width: 20 },
        { header: 'full_name', key: 'name', width: 30 },
        { header: 'hire_date', key: 'date', width: 20 },
        { header: 'department_name', key: 'dept', width: 25 },
    ];

    // Style the header row
    const headerRow = sheet.getRow(1);
    headerRow.font = { bold: true, color: { argb: 'FFFFFFFF' } };
    headerRow.fill = {
        type: 'pattern',
        pattern: 'solid',
        fgColor: { argb: 'FF10B981' } // Emerald 500
    };

    // Add a sample row
    sheet.addRow({
        id: 'NV001',
        name: 'Nguyễn Văn A',
        date: '2023-01-15',
        dept: 'Kỹ thuật'
    });
    
    sheet.addRow({
        id: 'NV002',
        name: 'Trần Thị B',
        date: '2023-02-01',
        dept: 'Nhân sự'
    });

    const dir = path.join(__dirname, 'public', 'templates');
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }

    const filePath = path.join(dir, 'employee_import_template.xlsx');
    await workbook.xlsx.writeFile(filePath);
    console.log(`Template generated at: ${filePath}`);
}

generateTemplate().catch(console.error);
